use crate::model::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct ServersFile {
    schema: u32,
    servers: Vec<Server>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct StatsFile {
    schema: u32,
    servers: BTreeMap<String, ServerStats>,
}

pub struct Store {
    dir: PathBuf,
}

pub fn engine_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("FoundryPerformance").join("engine")
}

impl Store {
    pub fn new(dir: PathBuf) -> Store {
        Store { dir }
    }

    pub fn default_dir() -> PathBuf {
        let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        base.join("FoundryPerformance")
    }

    /// `Ok(None)` — файла нет; `Err(notice)` — файл повреждён (переименован в .bak).
    fn read<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, Notice> {
        let path = self.dir.join(name);
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(Notice::new("notice.storeCorrupted").with("file", name)),
        };
        match serde_json::from_str(&text) {
            Ok(v) => Ok(Some(v)),
            Err(_) => {
                let _ = fs::rename(&path, self.dir.join(format!("{name}.bak")));
                Err(Notice::new("notice.storeCorrupted").with("file", name))
            }
        }
    }

    /// Запись через временный файл, чтобы сбой посреди записи не портил данные.
    fn write<T: Serialize>(&self, name: &str, value: &T) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join(format!("{name}.tmp"));
        let json = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
        fs::write(&tmp, json)?;
        fs::rename(&tmp, self.dir.join(name))
    }

    pub fn load_settings(&self) -> (Settings, Option<Notice>) {
        match self.read::<Settings>("settings.json") {
            Ok(Some(mut s)) => {
                s.schema = SCHEMA;
                (s, None)
            }
            Ok(None) => (Settings::default(), None),
            Err(n) => (Settings::default(), Some(n)),
        }
    }

    pub fn save_settings(&self, s: &Settings) -> io::Result<()> {
        self.write("settings.json", s)
    }

    pub fn load_servers(&self) -> (Vec<Server>, Option<Notice>) {
        match self.read::<ServersFile>("servers.json") {
            Ok(Some(f)) => (f.servers, None),
            Ok(None) => (Vec::new(), None),
            Err(n) => (Vec::new(), Some(n)),
        }
    }

    pub fn save_servers(&self, servers: &[Server]) -> io::Result<()> {
        self.write("servers.json", &ServersFile { schema: SCHEMA, servers: servers.to_vec() })
    }

    pub fn load_stats(&self) -> (BTreeMap<String, ServerStats>, Option<Notice>) {
        match self.read::<StatsFile>("stats.json") {
            Ok(Some(f)) => (f.servers, None),
            Ok(None) => (BTreeMap::new(), None),
            Err(n) => (BTreeMap::new(), Some(n)),
        }
    }

    pub fn save_stats(&self, stats: &BTreeMap<String, ServerStats>) -> io::Result<()> {
        self.write("stats.json", &StatsFile { schema: SCHEMA, servers: stats.clone() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> (tempfile::TempDir, Store) {
        let d = tempfile::tempdir().unwrap();
        let s = Store::new(d.path().to_path_buf());
        (d, s)
    }

    #[test]
    fn missing_settings_start_on_balance() {
        let (_d, s) = tmp();
        let (settings, notice) = s.load_settings();
        assert_eq!(settings.profile, ProfileId::Balance);
        assert!(notice.is_none());
    }

    #[test]
    fn settings_roundtrip() {
        let (_d, s) = tmp();
        let st = Settings {
            profile: ProfileId::Quality,
            engine: EngineSettings { extra_args: "--foo".into(), ..EngineSettings::default() },
            ..Settings::default()
        };
        s.save_settings(&st).unwrap();
        assert_eq!(s.load_settings().0, st);
    }

    #[test]
    fn corrupted_file_is_backed_up_and_reported() {
        let (d, s) = tmp();
        fs::write(d.path().join("servers.json"), "{not json").unwrap();
        let (servers, notice) = s.load_servers();
        assert!(servers.is_empty());
        let n = notice.unwrap();
        assert_eq!(n.key, "notice.storeCorrupted");
        assert_eq!(n.params["file"], "servers.json");
        assert!(d.path().join("servers.json.bak").exists());
    }

    #[test]
    fn servers_and_stats_roundtrip() {
        let (_d, s) = tmp();
        let srv = vec![Server { id: "a".into(), name: "A".into(), url: "https://a".into(), profile: None, overrides: Overrides::default(), game_url: None }];
        s.save_servers(&srv).unwrap();
        assert_eq!(s.load_servers().0, srv);

        let mut stats = BTreeMap::new();
        stats.insert("a".to_string(), ServerStats::default());
        s.save_stats(&stats).unwrap();
        assert_eq!(s.load_stats().0, stats);
    }

    #[test]
    fn save_creates_directory() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::new(d.path().join("nested"));
        s.save_settings(&Settings::default()).unwrap();
        assert!(d.path().join("nested/settings.json").exists());
    }
}
