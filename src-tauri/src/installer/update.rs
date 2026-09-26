//! Автообновление через GitHub Releases: лента `latest.json`, проверка SHA-256 и подписи
//! (`tauri signer`, формат minisign), замена exe через `selfreplace`.
use super::selfreplace;
use super::version::Version;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub const FEED_URL: &str = "https://github.com/TacticalOtaku/FoundryPerformance/releases/latest/download/latest.json";
/// Качаем только из релизов нашего репозитория — даже если ленту подменят.
pub const RELEASES_PREFIX: &str = "https://github.com/TacticalOtaku/FoundryPerformance/releases/download/";
/// Публичный ключ `tauri signer` (base64 текста .pub). Закрытый — только в секретах GitHub.
pub const PUBLIC_KEY: &str = include_str!("../../update.pub");

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

pub fn is_newer(current: Version, m: &Manifest) -> bool {
    Version::parse(&m.version).is_some_and(|v| v > current)
}

fn decode_text(b64: &str) -> Option<String> {
    String::from_utf8(STANDARD.decode(b64.trim()).ok()?).ok()
}

/// Все проверки перед заменой exe; ошибка — i18n-ключ.
pub fn verify(bytes: &[u8], m: &Manifest, public_key_b64: &str) -> Result<(), &'static str> {
    if !m.url.starts_with(RELEASES_PREFIX) {
        return Err("update.err.source");
    }
    if !format!("{:x}", Sha256::digest(bytes)).eq_ignore_ascii_case(m.sha256.trim()) {
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

fn client(timeout_s: u64) -> Option<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(timeout_s))
        .user_agent(concat!("FoundryPerformance/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()
}

/// Тихая проверка: нет сети или релизов — просто `None`.
pub async fn fetch_manifest() -> Option<Manifest> {
    let resp = client(8)?.get(FEED_URL).send().await.ok()?.error_for_status().ok()?;
    resp.json::<Manifest>().await.ok()
}

pub async fn download(url: &str, progress: impl Fn(u8)) -> Result<Vec<u8>, &'static str> {
    let fail = |_| "update.err.download";
    let mut resp = client(180).ok_or("update.err.download")?.get(url).send().await.map_err(fail)?.error_for_status().map_err(fail)?;
    let total = resp.content_length().unwrap_or(0);
    let mut buf = Vec::with_capacity(total as usize);
    let mut last = u8::MAX;
    while let Some(chunk) = resp.chunk().await.map_err(fail)? {
        buf.extend_from_slice(&chunk);
        // Без Content-Length процентов не будет — checked_div вернёт None
        if let Some(pct) = (buf.len() as u64 * 100).checked_div(total) {
            let pct = pct.min(100) as u8;
            if pct != last {
                last = pct;
                progress(pct);
            }
        }
    }
    Ok(buf)
}

/// Кладёт проверенный exe на место текущего (текущий остаётся рядом как `.old`).
pub fn apply(bytes: &[u8], exe: &Path) -> Result<(), &'static str> {
    let mut tmp = exe.as_os_str().to_owned();
    tmp.push(".download");
    let tmp = std::path::PathBuf::from(tmp);
    fs::write(&tmp, bytes).map_err(|_| "update.err.apply")?;
    let result = selfreplace::replace(&tmp, exe).map_err(|_| "update.err.apply");
    let _ = fs::remove_file(&tmp);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Настоящая подпись строки `hello` ключом проекта с `--app-version 0.2.0`.
    const HELLO_SIG: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVUVlRsRUhZNEhkS3ViWmNNTThwK3JBZzJNWldSdXRHS01RM1BIbkhwK0Vvbzc5a2E3eEhXOGI1andSNmxWMGUxNzlpTmNZdGUwMy9rZGgzWUJOKzZ3UWNCQUg3RGxKYlF3PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkwNDUyMDMwCWZpbGU6ZnAtc2lndGVzdC5iaW4JdmVyc2lvbjowLjIuMApCK3ptOTRtRDBNUFVPSXo1WmRQL0FhU0FoeTZjTXZDcVZuUDVPYVBRNHN6TXZ6MUlDMVlPaC9QRGVSb0RTU0pvSHZuU3pYN01seGRldis3TFYzV1lBQT09Cg==";
    const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    fn manifest() -> Manifest {
        Manifest {
            version: "0.2.0".into(),
            notes: String::new(),
            url: format!("{RELEASES_PREFIX}v0.2.0/FoundryPerformance-0.2.0.exe"),
            sha256: HELLO_SHA256.into(),
            signature: HELLO_SIG.into(),
        }
    }

    #[test]
    fn genuine_release_passes() {
        assert_eq!(verify(b"hello", &manifest(), PUBLIC_KEY), Ok(()));
    }

    #[test]
    fn foreign_source_is_rejected_first() {
        let m = Manifest { url: "https://evil.example/FoundryPerformance.exe".into(), ..manifest() };
        assert_eq!(verify(b"hello", &m, PUBLIC_KEY), Err("update.err.source"));
    }

    #[test]
    fn tampered_file_fails_hash() {
        assert_eq!(verify(b"hellO", &manifest(), PUBLIC_KEY), Err("update.err.hash"));
    }

    #[test]
    fn matching_hash_but_foreign_file_fails_signature() {
        let tampered = b"hellO";
        let m = Manifest { sha256: format!("{:x}", Sha256::digest(tampered)), ..manifest() };
        assert_eq!(verify(tampered, &m, PUBLIC_KEY), Err("update.err.signature"));
    }

    #[test]
    fn manifest_cannot_relabel_the_signed_version() {
        let m = Manifest { version: "0.3.0".into(), ..manifest() };
        assert_eq!(verify(b"hello", &m, PUBLIC_KEY), Err("update.err.version"));
    }

    #[test]
    fn newer_only_for_higher_valid_versions() {
        let cur = Version(0, 1, 0);
        assert!(is_newer(cur, &manifest()));
        assert!(!is_newer(cur, &Manifest { version: "0.1.0".into(), ..manifest() }));
        assert!(!is_newer(cur, &Manifest { version: "garbage".into(), ..manifest() }));
    }

    /// Сквозная проверка конвейера релиза на локальной сборке:
    /// `npm run dist` → `tauri signer sign --app-version` → `node scripts/make-manifest.mjs`,
    /// затем `cargo test local_release_verifies -- --ignored`.
    #[test]
    #[ignore]
    fn local_release_verifies() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../dist-release");
        let m: Manifest = serde_json::from_str(&fs::read_to_string(dir.join("latest.json")).unwrap()).unwrap();
        let exe = fs::read(dir.join(format!("FoundryPerformance-{}.exe", m.version))).unwrap();
        assert_eq!(verify(&exe, &m, PUBLIC_KEY), Ok(()));
    }

    #[test]
    fn manifest_parses_feed_json() {
        let m: Manifest = serde_json::from_str(r#"{"version":"0.2.0","url":"u","sha256":"s","signature":"g"}"#).unwrap();
        assert_eq!((m.version.as_str(), m.notes.as_str()), ("0.2.0", ""));
    }
}
