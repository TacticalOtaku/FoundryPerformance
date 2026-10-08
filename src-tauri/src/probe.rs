use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use url::{Host, Url};

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub reachable: bool,
    pub foundry: bool,
    pub active: bool,
    pub version: Option<String>,
    pub world: Option<String>,
    pub system: Option<String>,
    pub users: Option<u32>,
}

const FOUNDRY_ROUTES: [&str; 5] = ["game", "join", "setup", "auth", "players"];

pub fn normalize_url(input: &str) -> Result<Url, String> {
    let s = input.trim();
    if s.is_empty() || s.contains(char::is_whitespace) {
        return Err("err.urlInvalid".into());
    }
    let with_scheme = if s.contains("://") {
        s.to_string()
    } else {
        // Голый IP и localhost почти всегда без TLS (локальная сеть, Docker, Tailscale)
        let plain = Url::parse(&format!("http://{s}"))
            .is_ok_and(|u| matches!(u.host(), Some(Host::Ipv4(_) | Host::Ipv6(_))) || u.host_str() == Some("localhost"));
        format!("{}://{s}", if plain { "http" } else { "https" })
    };
    let mut url = Url::parse(&with_scheme).map_err(|_| "err.urlInvalid".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("err.urlInvalid".into());
    }
    // `/game`, `/join` и т.п. отбрасываем: храним базовый адрес сервера (с routePrefix, если он есть)
    let mut segs: Vec<String> = url
        .path_segments()
        .map(|p| p.filter(|x| !x.is_empty()).map(String::from).collect())
        .unwrap_or_default();
    if segs.last().is_some_and(|l| FOUNDRY_ROUTES.contains(&l.as_str())) {
        segs.pop();
    }
    let path = if segs.is_empty() { "/".to_string() } else { format!("/{}/", segs.join("/")) };
    url.set_path(&path);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

#[derive(Deserialize)]
struct RawStatus {
    active: Option<bool>,
    version: Option<String>,
    world: Option<String>,
    system: Option<String>,
    users: Option<u32>,
}

pub fn parse_status(body: &str) -> Option<ProbeResult> {
    let raw: RawStatus = serde_json::from_str(body).ok()?;
    let active = raw.active?;
    raw.version.as_ref()?;
    Some(ProbeResult { reachable: true, foundry: true, active, version: raw.version, world: raw.world, system: raw.system, users: raw.users })
}

/// Хостинги, где адрес слота — страница хостинга, а Foundry живёт по другому адресу.
/// Sqyre: `www.sqyre.app/games/<slug>/` → `<slug>.games.sqyre.app`.
pub fn hosted_foundry(url: &Url) -> Option<Url> {
    if !matches!(url.host_str()?, "www.sqyre.app" | "sqyre.app") {
        return None;
    }
    let segs: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
    let ["games", slug] = segs.as_slice() else { return None };
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return None;
    }
    Url::parse(&format!("https://{slug}.games.sqyre.app/")).ok()
}

/// Ответ `GET /game` без редиректов — когда хостинг закрыл `/api/status`.
/// Foundry отправляет гостя на `/join`, если мир запущен, и на `/setup` / `/auth`, если нет.
pub fn parse_game_redirect(status: u16, location: Option<&str>) -> ProbeResult {
    if status >= 500 {
        return ProbeResult::default();
    }
    let redirect = (300..400).contains(&status);
    let path = location.and_then(|l| l.split(['?', '#']).next()).map(|p| p.trim_end_matches('/'));
    let foundry = |active| ProbeResult { reachable: true, foundry: true, active, ..ProbeResult::default() };
    match path {
        Some(p) if redirect && p.ends_with("/join") => foundry(true),
        Some(p) if redirect && ["/setup", "/auth", "/license"].iter().any(|r| p.ends_with(r)) => foundry(false),
        _ => ProbeResult { reachable: true, ..ProbeResult::default() },
    }
}

/// Один клиент на всё приложение: он держит пул соединений, а лаунчер
/// перепроверяет серверы раз в минуту.
fn client() -> Option<&'static reqwest::Client> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT.get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(3)).build().ok()).as_ref()
}

/// Для `/game`: редирект и есть ответ, следовать ему не нужно.
fn bare_client() -> Option<&'static reqwest::Client> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(3)).redirect(reqwest::redirect::Policy::none()).build().ok())
        .as_ref()
}

pub async fn probe(input: &str) -> ProbeResult {
    let Ok(slot) = normalize_url(input) else { return ProbeResult::default() };
    let base = hosted_foundry(&slot).unwrap_or(slot);
    let (Ok(status_url), Ok(game_url)) = (base.join("api/status"), base.join("game")) else { return ProbeResult::default() };
    let (Some(client), Some(bare)) = (client(), bare_client()) else { return ProbeResult::default() };
    let Ok(resp) = client.get(status_url).send().await else { return ProbeResult::default() };
    if resp.status().is_server_error() {
        return ProbeResult::default();
    }
    if let Some(r) = parse_status(&resp.text().await.unwrap_or_default()) {
        return r;
    }
    // Хостинг закрыл /api/status — Foundry выдаёт себя редиректом с /game
    match bare.get(game_url).send().await {
        Err(_) => ProbeResult { reachable: true, ..ProbeResult::default() },
        Ok(r) => parse_game_redirect(r.status().as_u16(), r.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_adds_scheme_and_strips_foundry_routes() {
        assert_eq!(normalize_url("vtt.example.com").unwrap().as_str(), "https://vtt.example.com/");
        assert_eq!(normalize_url("  http://192.168.1.40:30000/game ").unwrap().as_str(), "http://192.168.1.40:30000/");
        assert_eq!(normalize_url("https://host/foundry/join").unwrap().as_str(), "https://host/foundry/");
        assert_eq!(normalize_url("localhost:30000").unwrap().as_str(), "http://localhost:30000/");
        // голый IP без схемы — http: Docker, Tailscale, любая локальная сеть
        assert_eq!(normalize_url("172.17.0.2:30000").unwrap().as_str(), "http://172.17.0.2:30000/");
        assert_eq!(normalize_url("100.101.1.2:30000/game").unwrap().as_str(), "http://100.101.1.2:30000/");
        assert_eq!(normalize_url("192.168.1.40:443").unwrap().as_str(), "http://192.168.1.40:443/");
        // домен, похожий на IP, остаётся на https
        assert_eq!(normalize_url("10.example.com").unwrap().as_str(), "https://10.example.com/");
    }

    #[test]
    fn normalize_rejects_garbage() {
        assert_eq!(normalize_url("").unwrap_err(), "err.urlInvalid");
        assert_eq!(normalize_url("ftp://x").unwrap_err(), "err.urlInvalid");
        assert_eq!(normalize_url("not a url at all").unwrap_err(), "err.urlInvalid");
    }

    #[test]
    fn parse_active_world() {
        let r = parse_status(r#"{"active":true,"version":"14.349","world":"strahd","system":"dnd5e","users":3}"#).unwrap();
        assert!(r.reachable && r.foundry && r.active);
        assert_eq!(r.version.as_deref(), Some("14.349"));
        assert_eq!(r.world.as_deref(), Some("strahd"));
        assert_eq!(r.users, Some(3));
    }

    #[test]
    fn parse_no_world_and_non_foundry() {
        let r = parse_status(r#"{"active":false,"version":"14.349"}"#).unwrap();
        assert!(r.foundry && !r.active);
        assert!(parse_status(r#"{"hello":"world"}"#).is_none());
        assert!(parse_status("<html>").is_none());
    }

    fn sqyre(u: &str) -> Option<String> {
        hosted_foundry(&normalize_url(u).unwrap()).map(|u| u.to_string())
    }

    #[test]
    fn sqyre_game_page_maps_to_its_foundry_host() {
        let want = Some("https://aldarionv210-3c11b9b6.games.sqyre.app/".to_string());
        assert_eq!(sqyre("https://www.sqyre.app/games/aldarionv210-3c11b9b6/"), want);
        assert_eq!(sqyre("https://www.sqyre.app/games/aldarionv210-3c11b9b6"), want);
        assert_eq!(sqyre("sqyre.app/games/aldarionv210-3c11b9b6"), want);
        assert_eq!(sqyre("https://vtt.example.com/games/x/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/games/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/games/x/extra/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/assets/x/"), None);
        assert_eq!(sqyre("https://www.sqyre.app/games/a.b/"), None);
    }

    #[test]
    fn game_redirect_reveals_foundry_behind_a_closed_status() {
        let active = parse_game_redirect(302, Some("/join"));
        assert!(active.reachable && active.foundry && active.active);
        assert!(parse_game_redirect(302, Some("https://x.games.sqyre.app/join?x=1")).active);
        for to in ["/setup", "/auth", "/license"] {
            let r = parse_game_redirect(302, Some(to));
            assert!(r.foundry && !r.active, "{to}");
        }
        assert_eq!(parse_game_redirect(502, None), ProbeResult::default());
        for (code, to) in [(200, None), (404, None), (302, Some("https://www.sqyre.app/games/x"))] {
            let r = parse_game_redirect(code, to);
            assert!(r.reachable && !r.foundry, "{code}");
        }
    }

    /// Вживую: `cargo test live_sqyre_slot_reports_foundry -- --ignored` (сервер должен работать).
    #[tokio::test]
    #[ignore]
    async fn live_sqyre_slot_reports_foundry() {
        let r = probe("https://www.sqyre.app/games/aldarionv210-3c11b9b6/").await;
        assert!(r.foundry, "{r:?}");
    }
}
