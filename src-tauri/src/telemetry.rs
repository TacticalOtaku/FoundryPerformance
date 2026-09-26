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
}

#[derive(Debug, Default, PartialEq)]
pub struct Effects {
    pub stats: bool,
    pub servers: bool,
    pub settings: bool,
    pub notice: Option<Notice>,
}

fn fps_ok(x: f32) -> bool {
    x.is_finite() && (0.0..=1000.0).contains(&x)
}

pub fn validate(r: &Report) -> bool {
    match r {
        Report::Session { avg, low1, .. } => fps_ok(*avg) && fps_ok(*low1),
        Report::Bench { avg, low1, min, .. } => fps_ok(*avg) && fps_ok(*low1) && fps_ok(*min),
        Report::ProfileChanged { .. } | Report::WebglLost { .. } => true,
        Report::FoundryUrl { url } => {
            url.len() <= 2048 && (url.starts_with("https://") || url.starts_with("http://")) && crate::probe::normalize_url(url).is_ok()
        }
    }
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
            stats.entry(server_id.to_string()).or_default().last_session = Some(FpsSummary { avg, low1, profile, at: now });
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
}
