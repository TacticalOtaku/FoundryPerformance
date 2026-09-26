use crate::model::EngineSettings;

/// При вызове `additional_browser_args` wry перестаёт передавать свои умолчания
/// `--disable-features=…`, поэтому они продублированы здесь.
const COMPAT: [&str; 3] = [
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
    "--allow-insecure-localhost",
    "--allow-running-insecure-content",
];

fn key(flag: &str) -> &str {
    flag.split('=').next().unwrap_or(flag)
}

pub fn build(engine: &EngineSettings) -> String {
    let mut flags: Vec<String> = vec![
        format!("--use-angle={}", engine.angle.flag_value()),
        "--enable-gpu-rasterization".into(),
        "--enable-zero-copy".into(),
        "--ignore-gpu-blocklist".into(),
        "--force-high-performance-gpu".into(),
        "--disable-renderer-backgrounding".into(),
        format!("--disk-cache-size={}", u64::from(engine.disk_cache_mb) * 1024 * 1024),
        "--js-flags=--max-old-space-size=4096".into(),
    ];
    flags.extend(COMPAT.iter().map(|s| s.to_string()));

    for extra in engine.extra_args.split_whitespace().filter(|a| a.starts_with("--")) {
        if let Some(i) = flags.iter().position(|f| key(f) == key(extra)) {
            flags.remove(i);
        }
        flags.push(extra.to_string());
    }
    flags.join(" ")
}

pub fn safe() -> String {
    COMPAT.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AngleBackend;

    #[test]
    fn default_flags_contain_performance_set() {
        let f = build(&EngineSettings::default());
        for flag in [
            "--use-angle=d3d11",
            "--enable-gpu-rasterization",
            "--enable-zero-copy",
            "--ignore-gpu-blocklist",
            "--force-high-performance-gpu",
            "--disable-renderer-backgrounding",
            "--disk-cache-size=2147483648",
            "--js-flags=--max-old-space-size=4096",
            "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection",
        ] {
            assert!(f.split(' ').any(|x| x == flag), "missing {flag} in {f}");
        }
    }

    #[test]
    fn angle_backend_is_configurable() {
        let e = EngineSettings { angle: AngleBackend::Vulkan, ..EngineSettings::default() };
        assert!(build(&e).contains("--use-angle=vulkan"));
    }

    #[test]
    fn extra_args_replace_same_key_and_append_new() {
        let e = EngineSettings { extra_args: "--use-angle=gl --foo --bad".into(), ..EngineSettings::default() };
        let f = build(&e);
        assert!(f.contains("--use-angle=gl"));
        assert!(!f.contains("--use-angle=d3d11"));
        // заменённый флаг переносится в конец, в порядке extra_args
        assert!(f.ends_with("--use-angle=gl --foo --bad"));
    }

    #[test]
    fn extra_args_ignore_non_flags() {
        let e = EngineSettings { extra_args: "rm -rf x".into(), ..EngineSettings::default() };
        let f = build(&e);
        assert!(!f.split(' ').any(|t| t == "rm" || t == "-rf" || t == "x"));
    }

    #[test]
    fn safe_mode_has_only_compat_flags() {
        let f = safe();
        assert!(!f.contains("--use-angle"));
        assert!(f.contains("--allow-insecure-localhost"));
    }
}
