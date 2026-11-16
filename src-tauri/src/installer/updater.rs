//! Глубокий модуль автообновления: конфигурация ленты, кэш манифеста, отмена,
//! скачивание, проверка SHA-256 + minisign и замена exe.
use super::install;
use super::Version;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

pub const FEED_URL: &str = "https://github.com/TacticalOtaku/FoundryPerformance/releases/latest/download/latest.json";
/// Качаем только из релизов нашего репозитория — даже если ленту подменят.
pub const RELEASES_PREFIX: &str = "https://github.com/TacticalOtaku/FoundryPerformance/releases/download/";
/// Публичный ключ `tauri signer` (base64 текста .pub). Закрытый — только в секретах GitHub.
pub const PUBLIC_KEY: &str = include_str!("../../update.pub");

/// Явная конфигурация ленты обновлений (без глобальных env-переменных и `cfg!` внутри логики).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedConfig {
    pub enabled: bool,
    pub feed_url: String,
    pub allowed_prefix: String,
    pub public_key_b64: String,
}

impl FeedConfig {
    /// В `tauri dev` GitHub не дёргаем на каждый перезапуск — кроме явной локальной ленты `FP_UPDATE_FEED`.
    pub fn from_env(dev: bool, env_feed: Option<&str>) -> Self {
        if let Some((feed_url, allowed_prefix)) = local_feed(dev, env_feed) {
            return Self { enabled: true, feed_url, allowed_prefix, public_key_b64: PUBLIC_KEY.to_string() };
        }
        Self {
            enabled: !dev,
            feed_url: FEED_URL.to_string(),
            allowed_prefix: RELEASES_PREFIX.to_string(),
            public_key_b64: PUBLIC_KEY.to_string(),
        }
    }

    pub fn production() -> Self {
        Self::from_env(cfg!(debug_assertions), std::env::var("FP_UPDATE_FEED").ok().as_deref())
    }
}

fn local_feed(dev: bool, env: Option<&str>) -> Option<(String, String)> {
    let feed = env.filter(|_| dev)?.trim();
    let dir = &feed[..feed.rfind('/')? + 1];
    Some((feed.to_string(), dir.to_string()))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    #[serde(default)]
    pub notes: String,
    pub url: String,
    pub sha256: String,
    /// base64 содержимого `.sig`, как его выдаёт `tauri signer sign`.
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDto {
    pub version: String,
    pub notes: String,
}

/// Аргументы нового exe после автообновления: исходные (`--portable`, `--open …`) плюс `--updated`.
/// Без них портативная сборка без маркера стартовала бы установщиком, а запуск по адресу терял бы адрес.
pub fn relaunch_args(original: impl Iterator<Item = String>) -> Vec<String> {
    let mut args: Vec<String> = original.filter(|a| a != super::UPDATED_FLAG).collect();
    args.push(super::UPDATED_FLAG.to_string());
    args
}

/// Сплэш ждёт ленту 4 с, фоновая проверка — 8 с.
pub fn check_timeout(requested: Option<u64>) -> u64 {
    requested.unwrap_or(8).clamp(1, 30)
}

pub fn is_newer(current: Version, m: &Manifest) -> bool {
    Version::parse(&m.version).is_some_and(|v| v > current)
}

/// SHA-256 в нижнем hex — в таком виде он лежит в `latest.json`.
fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn decode_text(b64: &str) -> Option<String> {
    String::from_utf8(STANDARD.decode(b64.trim()).ok()?).ok()
}

/// Все проверки перед заменой exe; ошибка — i18n-ключ.
pub fn verify(bytes: &[u8], m: &Manifest, allowed_prefix: &str, public_key_b64: &str) -> Result<(), &'static str> {
    if !m.url.starts_with(allowed_prefix) {
        return Err("update.err.source");
    }
    if !sha256_hex(bytes).eq_ignore_ascii_case(m.sha256.trim()) {
        return Err("update.err.hash");
    }
    let key = decode_text(public_key_b64).and_then(|t| PublicKey::decode(&t).ok()).ok_or("update.err.signature")?;
    let sig = decode_text(&m.signature).and_then(|t| Signature::decode(&t).ok()).ok_or("update.err.signature")?;
    key.verify(bytes, &sig, false).map_err(|_| "update.err.signature")?;
    // Версия в подписанном комментарии защищает от ленты, выдающей старый exe за новый
    let signed = sig.trusted_comment().split('\t').find_map(|f| f.strip_prefix("version:"));
    if signed != Some(m.version.as_str()) {
        return Err("update.err.version");
    }
    Ok(())
}

fn http_client(timeout_s: u64) -> Option<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_s))
        .user_agent(concat!("FoundryPerformance/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()
}

pub struct Updater {
    current: Version,
    feed: FeedConfig,
    manifest: Mutex<Option<Manifest>>,
    cancelled: AtomicBool,
    applying: AtomicBool,
}

impl Updater {
    pub fn new(current: Version, feed: FeedConfig) -> Self {
        Self {
            current,
            feed,
            manifest: Mutex::new(None),
            cancelled: AtomicBool::new(false),
            applying: AtomicBool::new(false),
        }
    }

    pub fn production() -> Self {
        Self::new(Version::current(), FeedConfig::production())
    }

    pub fn applying(&self) -> bool {
        self.applying.load(Ordering::SeqCst)
    }

    pub fn clear_applying(&self) {
        self.applying.store(false, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Проверяет ленту обновлений и кэширует найденный манифест (кнопка ставит ровно то, что показали).
    pub async fn check(&self, timeout_s: Option<u64>) -> Result<Option<UpdateDto>, &'static str> {
        if !self.feed.enabled {
            return Ok(None);
        }
        let fail = |_| "update.err.check";
        let resp = http_client(check_timeout(timeout_s))
            .ok_or("update.err.check")?
            .get(&self.feed.feed_url)
            .send()
            .await
            .map_err(fail)?
            .error_for_status()
            .map_err(fail)?;
        let m = resp.json::<Manifest>().await.map_err(fail)?;
        if !is_newer(self.current, &m) {
            return Ok(None);
        }
        let dto = UpdateDto { version: m.version.clone(), notes: m.notes.clone() };
        *self.manifest.lock().expect("update state poisoned") = Some(m);
        Ok(Some(dto))
    }

    /// Скачивает кэшированный релиз, проверяет целостность и подпись и заменяет `target_exe`.
    pub async fn apply(&self, target_exe: &Path, progress: impl Fn(u8)) -> Result<(), &'static str> {
        self.cancelled.store(false, Ordering::SeqCst);
        let m = self.manifest.lock().expect("update state poisoned").clone().ok_or("update.err.download")?;
        let bytes = self.download(&m.url, progress).await?;
        let result = self.install_downloaded(&bytes, &m, target_exe);
        if result.is_err() {
            self.clear_applying();
        }
        result
    }

    async fn download(&self, url: &str, progress: impl Fn(u8)) -> Result<Vec<u8>, &'static str> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err("update.err.cancelled");
        }
        let fail = |_| "update.err.download";
        let mut resp = http_client(180).ok_or("update.err.download")?.get(url).send().await.map_err(fail)?.error_for_status().map_err(fail)?;
        let total = resp.content_length().unwrap_or(0);
        let mut buf = Vec::with_capacity(total as usize);
        let mut last = u8::MAX;
        while let Some(chunk) = resp.chunk().await.map_err(fail)? {
            // «Пропустить» на сплэше: не докачиваем и не ставим
            if self.cancelled.load(Ordering::SeqCst) {
                return Err("update.err.cancelled");
            }
            buf.extend_from_slice(&chunk);
            // Без Content-Length процентов не будет — checked_div вернёт None
            if let Some(pct) = (buf.len() as u64 * 100).checked_div(total) {
                let pct = pct.min(100) as u8;
                if pct != last {
                    last = pct;
                    progress(pct);
                    if self.cancelled.load(Ordering::SeqCst) {
                        return Err("update.err.cancelled");
                    }
                }
            }
        }
        Ok(buf)
    }

    fn install_downloaded(&self, bytes: &[u8], m: &Manifest, target_exe: &Path) -> Result<(), &'static str> {
        if self.cancelled.load(Ordering::SeqCst) {
            return Err("update.err.cancelled");
        }
        self.applying.store(true, Ordering::SeqCst);
        verify(bytes, m, &self.feed.allowed_prefix, &self.feed.public_key_b64)?;
        if !is_newer(self.current, m) {
            return Err("update.err.version");
        }
        if self.cancelled.load(Ordering::SeqCst) {
            return Err("update.err.cancelled");
        }
        let mut tmp = target_exe.as_os_str().to_owned();
        tmp.push(".download");
        let tmp = std::path::PathBuf::from(tmp);
        fs::write(&tmp, bytes).map_err(|_| "update.err.apply")?;
        let result = install::replace_exe(&tmp, target_exe).map_err(|_| "update.err.apply");
        let _ = fs::remove_file(&tmp);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::sync::Arc;

    /// Настоящая подпись строки `hello` ключом проекта с `--app-version 0.2.0`.
    const HELLO_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVUVlRsRUhZNEhkS3ViWmNNTThwK3JBZzJNWldSdXRHS01RM1BIbkhwK0Vvbzc5a2E3eEhXOGI1andSNmxWMGUxNzlpTmNZdGUwMy9rZGgzWUJOKzZ3UWNCQUg3RGxKYlF3PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkwNDUyMDMwCWZpbGU6ZnAtc2lndGVzdC5iaW4JdmVyc2lvbjowLjIuMApCK3ptOTRtRDBNUFVPSXo1WmRQL0FhU0FoeTZjTXZDcVZuUDVPYVBRNHN6TXZ6MUlDMVlPaC9QRGVSb0RTU0pvSHZuU3pYN01seGRldis3TFYzV1lBQT09Cg==";
    const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    fn manifest() -> Manifest {
        Manifest {
            version: "0.2.0".into(),
            notes: "- Исправление".into(),
            url: format!("{RELEASES_PREFIX}v0.2.0/FoundryPerformance-0.2.0.exe"),
            sha256: HELLO_SHA256.into(),
            signature: HELLO_SIG.into(),
        }
    }

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn verify_rejects_foreign_source_tampered_hash_bad_sig_and_relabelled_version() {
        assert_eq!(verify(b"hello", &manifest(), RELEASES_PREFIX, PUBLIC_KEY), Ok(()));
        let foreign = Manifest { url: "https://evil.example/FoundryPerformance.exe".into(), ..manifest() };
        assert_eq!(verify(b"hello", &foreign, RELEASES_PREFIX, PUBLIC_KEY), Err("update.err.source"));
        assert_eq!(verify(b"hellO", &manifest(), RELEASES_PREFIX, PUBLIC_KEY), Err("update.err.hash"));

        let tampered = b"hellO";
        let wrong_sig = Manifest { sha256: sha256_hex(tampered), ..manifest() };
        assert_eq!(verify(tampered, &wrong_sig, RELEASES_PREFIX, PUBLIC_KEY), Err("update.err.signature"));

        let relabel = Manifest { version: "0.3.0".into(), ..manifest() };
        assert_eq!(verify(b"hello", &relabel, RELEASES_PREFIX, PUBLIC_KEY), Err("update.err.version"));
    }

    #[test]
    fn feed_config_relaunch_args_and_timeout_helpers() {
        let env = Some("http://127.0.0.1:8000/latest.json");
        assert!(!FeedConfig::from_env(true, None).enabled);
        assert!(FeedConfig::from_env(false, env).enabled);
        assert_eq!(FeedConfig::from_env(false, env).feed_url, FEED_URL);
        let dev_local = FeedConfig::from_env(true, env);
        assert!(dev_local.enabled);
        assert_eq!((dev_local.feed_url.as_str(), dev_local.allowed_prefix.as_str()), ("http://127.0.0.1:8000/latest.json", "http://127.0.0.1:8000/"));

        assert_eq!(relaunch_args(s(&["--portable", "--open", "https://vtt.example.com/"]).into_iter()), s(&["--portable", "--open", "https://vtt.example.com/", "--updated"]));
        assert_eq!(relaunch_args(s(&["--updated", "--portable"]).into_iter()), s(&["--portable", "--updated"]));
        assert_eq!((check_timeout(None), check_timeout(Some(4)), check_timeout(Some(0)), check_timeout(Some(600))), (8, 4, 1, 30));

        let m: Manifest = serde_json::from_str(r#"{"version":"0.2.0","url":"u","sha256":"s","signature":"g"}"#).unwrap();
        assert_eq!((m.version.as_str(), m.notes.as_str()), ("0.2.0", ""));
        assert!(is_newer(Version(0, 1, 0), &m));
        assert!(!is_newer(Version(0, 2, 0), &m));
    }

    #[tokio::test]
    async fn check_download_verify_and_apply_over_local_tcp_and_tempdir() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let base = format!("http://{addr}/");
        let manifest_json = serde_json::to_string(&Manifest {
            version: "0.2.0".into(),
            notes: "- Обновление".into(),
            url: format!("{base}FoundryPerformance-0.2.0.exe"),
            sha256: HELLO_SHA256.into(),
            signature: HELLO_SIG.into(),
        })
        .unwrap();

        let srv = std::thread::spawn(move || {
            for _ in 0..2 {
                let (mut sock, _) = listener.accept().unwrap();
                let mut req = [0u8; 2048];
                let n = sock.read(&mut req).unwrap();
                let head_txt = String::from_utf8_lossy(&req[..n]);
                let body: &[u8] = if head_txt.starts_with("GET /latest.json") {
                    manifest_json.as_bytes()
                } else {
                    b"hello"
                };
                let resp = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                sock.write_all(resp.as_bytes()).unwrap();
                sock.write_all(body).unwrap();
            }
        });

        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join(install::EXE_NAME);
        fs::write(&exe, b"old-exe").unwrap();

        let feed = FeedConfig {
            enabled: true,
            feed_url: format!("{base}latest.json"),
            allowed_prefix: base,
            public_key_b64: PUBLIC_KEY.to_string(),
        };
        let updater = Updater::new(Version(0, 1, 0), feed);

        let dto = updater.check(Some(4)).await.unwrap().unwrap();
        assert_eq!(dto, UpdateDto { version: "0.2.0".into(), notes: "- Обновление".into() });

        let pcts = Mutex::new(Vec::new());
        updater.apply(&exe, |p| pcts.lock().unwrap().push(p)).await.unwrap();
        srv.join().unwrap();

        assert_eq!(*pcts.lock().unwrap().last().unwrap(), 100);
        assert_eq!(fs::read(&exe).unwrap(), b"hello");
        assert_eq!(fs::read(d.path().join(format!("{}{}", install::EXE_NAME, install::EXE_OLD_SUFFIX))).unwrap(), b"old-exe");
    }

    #[tokio::test]
    async fn mid_download_cancellation_stops_and_preserves_target_exe() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let base = format!("http://{addr}/");
        let manifest_json = serde_json::to_string(&Manifest {
            version: "0.2.0".into(),
            notes: String::new(),
            url: format!("{base}FoundryPerformance-0.2.0.exe"),
            sha256: HELLO_SHA256.into(),
            signature: HELLO_SIG.into(),
        })
        .unwrap();
        let (tx_cancelled, rx_cancelled) = mpsc::channel::<()>();

        let srv = std::thread::spawn(move || {
            // Запрос 1: latest.json
            let (mut s1, _) = listener.accept().unwrap();
            let mut req = [0u8; 2048];
            let _ = s1.read(&mut req);
            let h1 = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", manifest_json.len());
            s1.write_all(h1.as_bytes()).unwrap();
            s1.write_all(manifest_json.as_bytes()).unwrap();
            drop(s1);

            // Запрос 2: отдаём первый кусок 2 байта из 5, ждём отмены и только потом шлём остаток
            let (mut s2, _) = listener.accept().unwrap();
            let _ = s2.read(&mut req);
            let _ = s2.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhe");
            let _ = s2.flush();
            let _ = rx_cancelled.recv_timeout(Duration::from_secs(5));
            let _ = s2.write_all(b"llo");
        });

        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join(install::EXE_NAME);
        fs::write(&exe, b"untouched").unwrap();

        let feed = FeedConfig {
            enabled: true,
            feed_url: format!("{base}latest.json"),
            allowed_prefix: base,
            public_key_b64: PUBLIC_KEY.to_string(),
        };
        let updater = Arc::new(Updater::new(Version(0, 1, 0), feed));
        updater.check(Some(4)).await.unwrap().unwrap();

        let u_cb = updater.clone();
        let tx = Mutex::new(Some(tx_cancelled));
        let err = updater
            .apply(&exe, move |_pct| {
                u_cb.cancel();
                if let Some(ch) = tx.lock().unwrap().take() {
                    let _ = ch.send(());
                }
            })
            .await
            .unwrap_err();
        let _ = srv.join();

        assert_eq!(err, "update.err.cancelled");
        assert!(!updater.applying());
        assert_eq!(fs::read(&exe).unwrap(), b"untouched");
    }

    #[test]
    fn cancelled_update_never_installs() {
        let updater = Updater::new(Version(0, 1, 0), FeedConfig::from_env(false, None));
        updater.cancel();
        let d = tempfile::tempdir().unwrap();
        assert_eq!(updater.install_downloaded(b"x", &manifest(), &d.path().join(install::EXE_NAME)), Err("update.err.cancelled"));
        assert!(!updater.applying());
    }
}
