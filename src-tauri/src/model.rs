use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProfileId {
    Quality,
    Balance,
    Potato,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoMode {
    Play,
    PauseUnfocused,
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrimeLevel {
    Soft,
    Medium,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngleBackend {
    #[serde(rename = "d3d11")]
    D3d11,
    #[serde(rename = "d3d11on12")]
    D3d11on12,
    #[serde(rename = "gl")]
    Gl,
    #[serde(rename = "vulkan")]
    Vulkan,
}

impl AngleBackend {
    pub fn flag_value(self) -> &'static str {
        match self {
            AngleBackend::D3d11 => "d3d11",
            AngleBackend::D3d11on12 => "d3d11on12",
            AngleBackend::Gl => "gl",
            AngleBackend::Vulkan => "vulkan",
        }
    }

    /// Порядок отката при отказе WebGL: d3d11 → d3d11on12 → gl → vulkan → d3d11.
    pub fn next(self) -> AngleBackend {
        match self {
            AngleBackend::D3d11 => AngleBackend::D3d11on12,
            AngleBackend::D3d11on12 => AngleBackend::Gl,
            AngleBackend::Gl => AngleBackend::Vulkan,
            AngleBackend::Vulkan => AngleBackend::D3d11,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Levers {
    pub perf_mode: u8,
    pub max_fps: u32,
    pub res_min: f32,
    pub res_max: f32,
    pub adaptive: bool,
    pub pixel_ratio_scaling: bool,
    pub light_animation: bool,
    pub vision_animation: bool,
    pub mipmap: bool,
    pub video: VideoMode,
    pub ui_blur: bool,
    pub sequencer: bool,
    pub fxmaster: bool,
    pub unfocused_fps: u32,
    pub prime: PrimeLevel,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Overrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perf_mode: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub res_min: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub res_max: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixel_ratio_scaling: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_animation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision_animation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mipmap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_blur: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequencer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fxmaster: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unfocused_fps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prime: Option<PrimeLevel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineSettings {
    pub angle: AngleBackend,
    pub disk_cache_mb: u32,
    pub extra_args: String,
}

impl Default for EngineSettings {
    fn default() -> Self {
        EngineSettings { angle: AngleBackend::D3d11, disk_cache_mb: 2048, extra_args: String::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Locale {
    #[default]
    Auto,
    Ru,
    En,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    Auto,
    Day,
    Night,
}

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub schema: u32,
    pub profile: ProfileId,
    pub overrides: Overrides,
    pub engine: EngineSettings,
    pub locale: Locale,
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema: SCHEMA,
            profile: ProfileId::Balance,
            overrides: Overrides::default(),
            engine: EngineSettings::default(),
            locale: Locale::Auto,
            theme: Theme::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub profile: Option<ProfileId>,
    #[serde(default)]
    pub overrides: Overrides,
    /// Настоящий адрес Foundry, если вход идёт через страницу хостинга (Sqyre и т.п.).
    /// Узнаётся от агента при первом входе в мир; по нему проверяется статус сервера.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FpsSummary {
    pub avg: f32,
    pub low1: f32,
    pub profile: ProfileId,
    pub at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchResult {
    pub avg: f32,
    pub low1: f32,
    pub min: f32,
    pub profile: ProfileId,
    pub at: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ServerStats {
    pub last_session: Option<FpsSummary>,
    pub last_bench: Option<BenchResult>,
}

/// Сообщение для ЖК лаунчера: ключ i18n + параметры.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notice {
    pub key: String,
    pub params: BTreeMap<String, String>,
}

impl Notice {
    pub fn new(key: &str) -> Notice {
        Notice { key: key.to_string(), params: BTreeMap::new() }
    }

    pub fn with(mut self, k: &str, v: impl Into<String>) -> Notice {
        self.params.insert(k.to_string(), v.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_serialize_as_camel_case_strings() {
        assert_eq!(serde_json::to_string(&ProfileId::Potato).unwrap(), "\"potato\"");
        assert_eq!(serde_json::to_string(&VideoMode::PauseUnfocused).unwrap(), "\"pauseUnfocused\"");
        assert_eq!(serde_json::to_string(&AngleBackend::D3d11on12).unwrap(), "\"d3d11on12\"");
    }

    #[test]
    fn angle_fallback_cycles_through_all_backends() {
        let mut a = AngleBackend::D3d11;
        let mut seen = vec![a];
        for _ in 0..3 {
            a = a.next();
            seen.push(a);
        }
        assert_eq!(seen, vec![AngleBackend::D3d11, AngleBackend::D3d11on12, AngleBackend::Gl, AngleBackend::Vulkan]);
        assert_eq!(a.next(), AngleBackend::D3d11);
    }

    #[test]
    fn settings_tolerate_missing_fields() {
        let s: Settings = serde_json::from_str(r#"{"profile":"potato"}"#).unwrap();
        assert_eq!(s.profile, ProfileId::Potato);
        assert_eq!(s.engine.angle, AngleBackend::D3d11);
        assert_eq!(s.schema, SCHEMA);
    }

    #[test]
    fn empty_overrides_serialize_to_empty_object() {
        assert_eq!(serde_json::to_string(&Overrides::default()).unwrap(), "{}");
    }
}
