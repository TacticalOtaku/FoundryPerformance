use crate::gpu::{self, GpuInfo};
use crate::model::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Report {
    Session { avg: f32, low1: f32, profile: ProfileId },
    Bench { avg: f32, low1: f32, min: f32, profile: ProfileId },
    ProfileChanged { profile: ProfileId },
    WebglLost { early: bool },
    /// Агент загрузился в мир: это настоящий адрес Foundry (для хостингов он отличается от адреса входа).
    FoundryUrl { url: String },
    /// Строка `UNMASKED_RENDERER_WEBGL` игрового канваса — на чём игра рисует на самом деле.
    Gpu { renderer: String },
}

#[derive(Debug, Default, PartialEq)]
pub struct Effects {
    pub stats: bool,
    pub servers: bool,
    pub settings: bool,
    pub notice: Option<Notice>,
    /// Сделан откат ANGLE — второй за сессию не нужен.
    pub fallback: bool,
}

fn fps_ok(x: f32) -> bool {
    x.is_finite() && (0.0..=1000.0).contains(&x)
}

pub fn validate(r: &Report) -> bool {
    match r {
        Report::Session { avg, low1, .. } => fps_ok(*avg) && fps_ok(*low1),
        Report::Bench { avg, low1, min, .. } => fps_ok(*avg) && fps_ok(*low1) && fps_ok(*min),
        Report::ProfileChanged { .. } | Report::WebglLost { .. } => true,
        Report::Gpu { renderer } => renderer.len() <= 512,
        Report::FoundryUrl { url } => {
            url.len() <= 2048 && (url.starts_with("https://") || url.starts_with("http://")) && crate::probe::normalize_url(url).is_ok()
        }
    }
}

/// Откуда в этой сессии ждём отчёты: адрес входа и уже известный адрес игры.
pub fn session_origins(server: &Server) -> Vec<url::Origin> {
    let mut out: Vec<url::Origin> = Vec::new();
    for u in [Some(server.url.as_str()), server.game_url.as_deref()].into_iter().flatten() {
        let Ok(u) = crate::probe::normalize_url(u) else { continue };
        // Страница игры хостинга доверенная сразу — проба её /api/status может быть закрыта
        for o in [Some(u.origin()), crate::probe::hosted_foundry(&u).map(|h| h.origin())].into_iter().flatten() {
            if !out.contains(&o) {
                out.push(o);
            }
        }
    }
    out
}

pub fn page_trusted(page: &url::Url, origins: &[url::Origin]) -> bool {
    origins.contains(&page.origin())
}

/// Страница может сообщить только свой собственный адрес.
pub fn describes_page(reported: &str, page: &url::Url) -> bool {
    crate::probe::normalize_url(reported).is_ok_and(|u| u.origin() == page.origin())
}

pub fn apply(
    report: &Report,
    server_id: &str,
    now: u64,
    stats: &mut BTreeMap<String, ServerStats>,
    servers: &mut [Server],
    settings: &mut Settings,
) -> Effects {
    let mut fx = Effects::default();
    match *report {
        Report::Session { avg, low1, profile } => {
            let s = stats.entry(server_id.to_string()).or_default();
            s.last_session = Some(FpsSummary { avg, low1, profile, at: now });
            s.history.push(avg);
            let excess = s.history.len().saturating_sub(HISTORY_LEN);
            s.history.drain(..excess);
            fx.stats = true;
        }
        Report::Bench { avg, low1, min, profile } => {
            stats.entry(server_id.to_string()).or_default().last_bench = Some(BenchResult { avg, low1, min, profile, at: now });
            fx.stats = true;
        }
        Report::ProfileChanged { profile } => {
            if let Some(s) = servers.iter_mut().find(|s| s.id == server_id) {
                s.profile = Some(profile);
                fx.servers = true;
            }
        }
        Report::WebglLost { early: true } => {
            let from = settings.engine.angle;
            settings.engine.angle = from.next();
            fx.settings = true;
            fx.notice = Some(
                Notice::new("notice.engineFallback")
                    .with("from", from.flag_value())
                    .with("to", settings.engine.angle.flag_value()),
            );
        }
        Report::WebglLost { early: false } => {}
        // Нужны адаптеры и режим запуска — отдельный путь `apply_gpu`
        Report::Gpu { .. } => {}
        Report::FoundryUrl { ref url } => {
            let Some(s) = servers.iter_mut().find(|s| s.id == server_id) else { return fx };
            let (Ok(game), Ok(slot)) = (crate::probe::normalize_url(url), crate::probe::normalize_url(&s.url)) else { return fx };
            // Прямой адрес Foundry в слоте — отдельный адрес игры не нужен
            let game = (game != slot).then(|| game.to_string());
            if s.game_url != game {
                s.game_url = game;
                fx.servers = true;
            }
        }
    }
    fx
}

/// Отчёт о рендерере игры. `persist` — обычный запуск (наши флаги), а не базовый замер;
/// `fallback_allowed` — откат ANGLE в этой сессии ещё не делали.
pub fn apply_gpu(renderer: &str, adapters: &[GpuInfo], persist: bool, fallback_allowed: bool, settings: &mut Settings) -> (Verdict, Effects) {
    let verdict = gpu::classify(renderer, adapters);
    let mut fx = Effects::default();
    if !persist {
        return (verdict, fx);
    }
    settings.gpu_check = Some(GpuCheck {
        verdict: verdict.clone(),
        renderer: renderer.to_string(),
        angle: settings.engine.angle,
        adapter: gpu::best(adapters).map(|a| a.name.clone()).unwrap_or_default(),
    });
    fx.settings = true;
    if verdict == Verdict::Software && fallback_allowed {
        let from = settings.engine.angle;
        settings.engine.angle = from.next();
        fx.fallback = true;
        fx.notice = Some(
            Notice::new("notice.softwareRender")
                .with("from", from.flag_value())
                .with("to", settings.engine.angle.flag_value()),
        );
    }
    (verdict, fx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn srv(id: &str) -> Server {
        Server { id: id.into(), name: "S".into(), url: "https://s".into(), profile: None, overrides: Overrides::default(), game_url: None }
    }

    #[test]
    fn parses_agent_json() {
        let r: Report = serde_json::from_str(r#"{"kind":"bench","avg":41.2,"low1":20.5,"min":12.0,"profile":"potato"}"#).unwrap();
        assert_eq!(r, Report::Bench { avg: 41.2, low1: 20.5, min: 12.0, profile: ProfileId::Potato });
        let r: Report = serde_json::from_str(r#"{"kind":"webglLost","early":true}"#).unwrap();
        assert_eq!(r, Report::WebglLost { early: true });
    }

    #[test]
    fn rejects_non_finite_or_absurd_numbers() {
        assert!(!validate(&Report::Session { avg: f32::NAN, low1: 1.0, profile: ProfileId::Balance }));
        assert!(!validate(&Report::Bench { avg: 5000.0, low1: 1.0, min: 1.0, profile: ProfileId::Balance }));
        assert!(validate(&Report::Session { avg: 58.0, low1: 31.0, profile: ProfileId::Balance }));
    }

    #[test]
    fn session_and_bench_update_stats() {
        let (mut stats, mut servers, mut settings) = (BTreeMap::new(), vec![srv("a")], Settings::default());
        let fx = apply(&Report::Session { avg: 50.0, low1: 30.0, profile: ProfileId::Balance }, "a", 7, &mut stats, &mut servers, &mut settings);
        assert!(fx.stats);
        assert_eq!(stats["a"].last_session.unwrap().at, 7);
        apply(&Report::Bench { avg: 40.0, low1: 20.0, min: 10.0, profile: ProfileId::Potato }, "a", 8, &mut stats, &mut servers, &mut settings);
        assert_eq!(stats["a"].last_bench.unwrap().min, 10.0);
    }

    #[test]
    fn session_history_keeps_the_last_eight_readings() {
        let (mut stats, mut servers, mut settings) = (BTreeMap::new(), vec![srv("a")], Settings::default());
        for i in 0..10 {
            let r = Report::Session { avg: i as f32, low1: 1.0, profile: ProfileId::Balance };
            apply(&r, "a", i, &mut stats, &mut servers, &mut settings);
        }
        assert_eq!(stats["a"].history, vec![2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);
    }

    #[test]
    fn profile_change_persists_on_server() {
        let (mut stats, mut servers, mut settings) = (BTreeMap::new(), vec![srv("a")], Settings::default());
        let fx = apply(&Report::ProfileChanged { profile: ProfileId::Potato }, "a", 0, &mut stats, &mut servers, &mut settings);
        assert!(fx.servers);
        assert_eq!(servers[0].profile, Some(ProfileId::Potato));
        let fx = apply(&Report::ProfileChanged { profile: ProfileId::Potato }, "adhoc", 0, &mut stats, &mut servers, &mut settings);
        assert!(!fx.servers);
    }

    #[test]
    fn foundry_url_is_remembered_only_when_it_differs_from_the_slot() {
        let mut servers = vec![Server { url: "https://www.sqyre.app/games/x/".into(), ..srv("a") }];
        let (mut stats, mut settings) = (BTreeMap::new(), Settings::default());
        let report = Report::FoundryUrl { url: "https://x.sqyre.app/game".into() };
        let fx = apply(&report, "a", 0, &mut stats, &mut servers, &mut settings);
        assert!(fx.servers);
        assert_eq!(servers[0].game_url.as_deref(), Some("https://x.sqyre.app/"));
        // тот же адрес повторно — ничего не пишем
        assert!(!apply(&report, "a", 0, &mut stats, &mut servers, &mut settings).servers);

        let mut direct = vec![Server { url: "https://vtt.example.com/".into(), ..srv("b") }];
        let same = Report::FoundryUrl { url: "https://vtt.example.com/game".into() };
        assert!(!apply(&same, "b", 0, &mut stats, &mut direct, &mut settings).servers);
        assert_eq!(direct[0].game_url, None);
    }

    #[test]
    fn foundry_url_must_be_http() {
        assert!(validate(&Report::FoundryUrl { url: "https://x.sqyre.app/game".into() }));
        assert!(!validate(&Report::FoundryUrl { url: "javascript:alert(1)".into() }));
        assert!(!validate(&Report::FoundryUrl { url: "x".repeat(3000) }));
    }

    #[test]
    fn early_webgl_loss_switches_angle_backend() {
        let (mut stats, mut servers, mut settings) = (BTreeMap::new(), vec![srv("a")], Settings::default());
        let fx = apply(&Report::WebglLost { early: true }, "a", 0, &mut stats, &mut servers, &mut settings);
        assert!(fx.settings);
        assert_eq!(settings.engine.angle, AngleBackend::D3d11on12);
        let n = fx.notice.unwrap();
        assert_eq!((n.key.as_str(), n.params["to"].as_str()), ("notice.engineFallback", "d3d11on12"));
        let fx = apply(&Report::WebglLost { early: false }, "a", 0, &mut stats, &mut servers, &mut settings);
        assert_eq!(fx, Effects::default());
    }

    #[test]
    fn reports_are_trusted_only_from_session_pages() {
        let mut s = srv("a");
        s.url = "https://www.sqyre.app/games/x/".into();
        s.game_url = Some("https://x.sqyre.app/".into());
        let origins = session_origins(&s);
        assert_eq!(origins.len(), 3); // + выведенный хост Sqyre
        assert!(page_trusted(&"https://x.sqyre.app/game".parse().unwrap(), &origins));
        assert!(page_trusted(&"https://www.sqyre.app/games/x/".parse().unwrap(), &origins));
        assert!(!page_trusted(&"https://evil.example/x".parse().unwrap(), &origins));
        assert!(!page_trusted(&"http://x.sqyre.app/game".parse().unwrap(), &origins));
    }

    const NV: &str = "ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 6GB (0x00001C03) Direct3D11 vs_5_0 ps_5_0, D3D11)";
    const SWIFT: &str = "ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)";

    fn nv() -> GpuInfo {
        GpuInfo { name: "NVIDIA GeForce GTX 1060 6GB".into(), vram_mb: 6144, vendor_id: 0x10DE }
    }

    #[test]
    fn parses_gpu_report_and_limits_its_length() {
        let r: Report = serde_json::from_str(r#"{"kind":"gpu","renderer":"ANGLE (NVIDIA, x)"}"#).unwrap();
        assert_eq!(r, Report::Gpu { renderer: "ANGLE (NVIDIA, x)".into() });
        assert!(validate(&r));
        assert!(!validate(&Report::Gpu { renderer: "x".repeat(513) }));
    }

    #[test]
    fn hardware_check_is_remembered() {
        let mut settings = Settings::default();
        let (v, fx) = apply_gpu(NV, &[nv()], true, true, &mut settings);
        assert_eq!(v, Verdict::Hardware { backend: Some(AngleBackend::D3d11) });
        assert!(fx.settings && !fx.fallback && fx.notice.is_none());
        let c = settings.gpu_check.unwrap();
        assert_eq!((c.renderer.as_str(), c.angle, c.adapter.as_str()), (NV, AngleBackend::D3d11, "NVIDIA GeForce GTX 1060 6GB"));
    }

    #[test]
    fn software_render_switches_the_backend_once_per_session() {
        let mut settings = Settings::default();
        let (v, fx) = apply_gpu(SWIFT, &[nv()], true, true, &mut settings);
        assert_eq!(v, Verdict::Software);
        assert!(fx.fallback && fx.settings);
        assert_eq!(settings.engine.angle, AngleBackend::D3d11on12);
        let n = fx.notice.unwrap();
        assert_eq!((n.key.as_str(), n.params["to"].as_str()), ("notice.softwareRender", "d3d11on12"));
        // проверка записана с бэкендом, на котором её сделали: после отката она уже не текущая
        assert_eq!(settings.gpu_check.as_ref().unwrap().angle, AngleBackend::D3d11);

        let (_, fx) = apply_gpu(SWIFT, &[nv()], true, false, &mut settings);
        assert!(!fx.fallback && fx.notice.is_none());
        assert_eq!(settings.engine.angle, AngleBackend::D3d11on12);
    }

    #[test]
    fn baseline_run_only_answers() {
        let mut settings = Settings::default();
        let (v, fx) = apply_gpu(SWIFT, &[nv()], false, true, &mut settings);
        assert_eq!(v, Verdict::Software);
        assert_eq!(fx, Effects::default());
        assert_eq!(settings, Settings::default());
    }

    #[test]
    fn sqyre_game_host_is_trusted_from_launch() {
        let s = Server { url: "https://www.sqyre.app/games/aldarionv210-3c11b9b6/".into(), ..srv("a") };
        let origins = session_origins(&s);
        assert!(page_trusted(&"https://aldarionv210-3c11b9b6.games.sqyre.app/game".parse().unwrap(), &origins));
        assert!(!page_trusted(&"https://other.games.sqyre.app/game".parse().unwrap(), &origins));
    }

    #[test]
    fn a_page_can_only_report_its_own_address() {
        let page: url::Url = "https://x.sqyre.app/game?session=1".parse().unwrap();
        assert!(describes_page("https://x.sqyre.app/game", &page));
        assert!(!describes_page("https://evil.example/", &page));
        assert!(!describes_page("not a url", &page));
    }
}
