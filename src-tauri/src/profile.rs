use crate::model::*;
use std::collections::BTreeMap;

pub fn preset(id: ProfileId) -> Levers {
    match id {
        ProfileId::Quality => Levers {
            perf_mode: 2,
            max_fps: 60,
            res_min: 1.0,
            res_max: 1.0,
            adaptive: false,
            pixel_ratio_scaling: true,
            light_animation: true,
            vision_animation: true,
            mipmap: true,
            video: VideoMode::Play,
            ui_blur: true,
            sequencer: true,
            fxmaster: true,
            unfocused_fps: 60,
            prime: PrimeLevel::Soft,
        },
        ProfileId::Balance => Levers {
            perf_mode: 1,
            max_fps: 60,
            res_min: 0.7,
            res_max: 1.0,
            adaptive: true,
            pixel_ratio_scaling: false,
            light_animation: true,
            vision_animation: false,
            mipmap: true,
            video: VideoMode::PauseUnfocused,
            ui_blur: false,
            sequencer: true,
            fxmaster: true,
            unfocused_fps: 15,
            prime: PrimeLevel::Medium,
        },
        ProfileId::Potato => Levers {
            perf_mode: 0,
            max_fps: 45,
            res_min: 0.55,
            res_max: 0.85,
            adaptive: true,
            pixel_ratio_scaling: false,
            light_animation: false,
            vision_animation: false,
            mipmap: false,
            video: VideoMode::Static,
            ui_blur: false,
            sequencer: false,
            fxmaster: false,
            unfocused_fps: 10,
            prime: PrimeLevel::Aggressive,
        },
    }
}

pub fn presets() -> BTreeMap<ProfileId, Levers> {
    [ProfileId::Quality, ProfileId::Balance, ProfileId::Potato]
        .into_iter()
        .map(|id| (id, preset(id)))
        .collect()
}

pub fn sanitize(mut l: Levers) -> Levers {
    l.perf_mode = l.perf_mode.min(3);
    l.max_fps = l.max_fps.clamp(10, 240);
    l.unfocused_fps = l.unfocused_fps.clamp(5, 240);
    l.res_min = l.res_min.clamp(0.25, 1.0);
    l.res_max = l.res_max.clamp(0.25, 1.0);
    if l.res_min > l.res_max {
        std::mem::swap(&mut l.res_min, &mut l.res_max);
    }
    l
}

pub fn apply(mut b: Levers, o: &Overrides) -> Levers {
    if let Some(v) = o.perf_mode { b.perf_mode = v; }
    if let Some(v) = o.max_fps { b.max_fps = v; }
    if let Some(v) = o.res_min { b.res_min = v; }
    if let Some(v) = o.res_max { b.res_max = v; }
    if let Some(v) = o.adaptive { b.adaptive = v; }
    if let Some(v) = o.pixel_ratio_scaling { b.pixel_ratio_scaling = v; }
    if let Some(v) = o.light_animation { b.light_animation = v; }
    if let Some(v) = o.vision_animation { b.vision_animation = v; }
    if let Some(v) = o.mipmap { b.mipmap = v; }
    if let Some(v) = o.video { b.video = v; }
    if let Some(v) = o.ui_blur { b.ui_blur = v; }
    if let Some(v) = o.sequencer { b.sequencer = v; }
    if let Some(v) = o.fxmaster { b.fxmaster = v; }
    if let Some(v) = o.unfocused_fps { b.unfocused_fps = v; }
    if let Some(v) = o.prime { b.prime = v; }
    sanitize(b)
}

pub fn active_id(settings: &Settings, server: Option<&Server>) -> ProfileId {
    server.and_then(|s| s.profile).unwrap_or(settings.profile)
}

/// Все три профиля с наложенными глобальными и серверными правками —
/// агенту нужны все, чтобы переключать профиль прямо в игре.
pub fn resolve_all(settings: &Settings, server: Option<&Server>) -> BTreeMap<ProfileId, Levers> {
    presets()
        .into_iter()
        .map(|(id, base)| {
            let mut l = apply(base, &settings.overrides);
            if let Some(s) = server {
                l = apply(l, &s.overrides);
            }
            (id, l)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(profile: Option<ProfileId>, overrides: Overrides) -> Server {
        Server { id: "s1".into(), name: "S".into(), url: "https://x".into(), profile, overrides }
    }

    #[test]
    fn presets_match_spec_table() {
        let q = preset(ProfileId::Quality);
        assert_eq!((q.perf_mode, q.max_fps, q.res_min, q.res_max, q.adaptive), (2, 60, 1.0, 1.0, false));
        let b = preset(ProfileId::Balance);
        assert_eq!((b.perf_mode, b.res_min, b.res_max, b.adaptive, b.vision_animation), (1, 0.7, 1.0, true, false));
        assert_eq!((b.video, b.ui_blur, b.unfocused_fps), (VideoMode::PauseUnfocused, false, 15));
        let p = preset(ProfileId::Potato);
        assert_eq!((p.perf_mode, p.max_fps, p.res_min, p.res_max), (0, 45, 0.55, 0.85));
        assert_eq!((p.light_animation, p.mipmap, p.video, p.sequencer, p.fxmaster), (false, false, VideoMode::Static, false, false));
    }

    #[test]
    fn server_profile_wins_over_global() {
        let s = Settings { profile: ProfileId::Quality, ..Settings::default() };
        assert_eq!(active_id(&s, Some(&server(Some(ProfileId::Potato), Overrides::default()))), ProfileId::Potato);
        assert_eq!(active_id(&s, Some(&server(None, Overrides::default()))), ProfileId::Quality);
        assert_eq!(active_id(&s, None), ProfileId::Quality);
    }

    #[test]
    fn server_overrides_apply_after_global_overrides() {
        let s = Settings {
            overrides: Overrides { max_fps: Some(30), mipmap: Some(false), ..Overrides::default() },
            ..Settings::default()
        };
        let srv = server(None, Overrides { max_fps: Some(50), ..Overrides::default() });
        let all = resolve_all(&s, Some(&srv));
        let b = all[&ProfileId::Balance];
        assert_eq!(b.max_fps, 50);
        assert!(!b.mipmap);
        assert_eq!(all[&ProfileId::Quality].max_fps, 50);
    }

    #[test]
    fn sanitize_clamps_values() {
        let l = sanitize(Levers { perf_mode: 9, max_fps: 1, res_min: 1.4, res_max: 0.1, unfocused_fps: 0, ..preset(ProfileId::Balance) });
        assert_eq!(l.perf_mode, 3);
        assert_eq!(l.max_fps, 10);
        assert_eq!(l.unfocused_fps, 5);
        assert!(l.res_min >= 0.25 && l.res_max <= 1.0 && l.res_min <= l.res_max);
    }

    #[test]
    fn presets_contain_all_three() {
        assert_eq!(presets().len(), 3);
    }
}
