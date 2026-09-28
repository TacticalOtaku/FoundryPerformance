use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
use url::Url;

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
    } else if s.starts_with("localhost") || s.starts_with("127.") || s.starts_with("192.168.") || s.starts_with("10.") {
        format!("http://{s}")
    } else {
        format!("https://{s}")
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

/// Один клиент на всё приложение: он держит пул соединений, а лаунчер
/// перепроверяет серверы раз в минуту.
fn client() -> Option<&'static reqwest::Client> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT.get_or_init(|| reqwest::Client::builder().timeout(Duration::from_secs(3)).build().ok()).as_ref()
}

pub async fn probe(input: &str) -> ProbeResult {
    let Ok(base) = normalize_url(input) else { return ProbeResult::default() };
    let Ok(status_url) = base.join("api/status") else { return ProbeResult::default() };
    let Some(client) = client() else { return ProbeResult::default() };
    match client.get(status_url).send().await {
        Err(_) => ProbeResult::default(),
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            parse_status(&body).unwrap_or(ProbeResult { reachable: true, ..ProbeResult::default() })
        }
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
}
