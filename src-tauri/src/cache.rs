//! Очистка кэша игры: скачанные файлы, скомпилированные скрипты и шейдеры.
//! Куки (вход в миры) и Local Storage (клиентские настройки Foundry) не трогаем.
use std::fs;
use std::path::Path;

/// Папки внутри каталога движка (`engine`), которые безопасно удалить целиком.
pub const CACHE_DIRS: &[&str] = &[
    "EBWebView/Default/Cache",
    "EBWebView/Default/Code Cache",
    "EBWebView/Default/GPUCache",
    "EBWebView/Default/DawnGraphiteCache",
    "EBWebView/Default/DawnWebGPUCache",
    "EBWebView/Default/Service Worker/CacheStorage",
    "EBWebView/Default/Service Worker/ScriptCache",
    "EBWebView/ShaderCache",
    "EBWebView/GrShaderCache",
    "EBWebView/GPUPersistentCache",
];

/// Метка «дочистить при следующем запуске», если WebView2 ещё держал файлы.
const PENDING: &str = "clear-cache.pending";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cleared {
    pub freed: u64,
    /// Часть файлов была занята — удалим при следующем запуске программы.
    pub pending: bool,
}

fn dir_size(p: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(p) else { return 0 };
    entries
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => dir_size(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0,
        })
        .sum()
}

pub fn size(engine: &Path) -> u64 {
    CACHE_DIRS.iter().map(|d| dir_size(&engine.join(d))).sum()
}

pub fn clear(engine: &Path) -> Cleared {
    let before = size(engine);
    let mut pending = false;
    for d in CACHE_DIRS {
        let p = engine.join(d);
        if p.exists() && fs::remove_dir_all(&p).is_err() {
            pending = true;
        }
    }
    if pending {
        let _ = fs::create_dir_all(engine);
        let _ = fs::write(engine.join(PENDING), b"");
    }
    Cleared { freed: before.saturating_sub(size(engine)), pending }
}

/// При старте, до создания окна игры: дочищаем то, что в прошлый раз было занято.
pub fn finish_pending(engine: &Path) {
    let marker = engine.join(PENDING);
    if marker.exists() && !clear(engine).pending {
        let _ = fs::remove_file(marker);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("fp-cache-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("EBWebView/Default/Cache/Cache_Data")).unwrap();
        fs::write(dir.join("EBWebView/Default/Cache/Cache_Data/f_0001"), vec![0u8; 3000]).unwrap();
        fs::create_dir_all(dir.join("EBWebView/GrShaderCache")).unwrap();
        fs::write(dir.join("EBWebView/GrShaderCache/data_0"), vec![0u8; 500]).unwrap();
        fs::create_dir_all(dir.join("EBWebView/Default/Local Storage/leveldb")).unwrap();
        fs::write(dir.join("EBWebView/Default/Local Storage/leveldb/000003.log"), b"settings").unwrap();
        fs::create_dir_all(dir.join("EBWebView/Default/Network")).unwrap();
        fs::write(dir.join("EBWebView/Default/Network/Cookies"), b"session").unwrap();
        dir
    }

    #[test]
    fn measures_only_cache_folders() {
        let e = engine("size");
        assert_eq!(size(&e), 3500);
        let _ = fs::remove_dir_all(e);
    }

    #[test]
    fn clears_caches_but_keeps_logins_and_settings() {
        let e = engine("clear");
        assert_eq!(clear(&e), Cleared { freed: 3500, pending: false });
        assert_eq!(size(&e), 0);
        assert!(e.join("EBWebView/Default/Network/Cookies").exists());
        assert!(e.join("EBWebView/Default/Local Storage/leveldb/000003.log").exists());
        assert!(!e.join(PENDING).exists());
        let _ = fs::remove_dir_all(e);
    }

    #[test]
    fn missing_engine_folder_is_fine() {
        let e = std::env::temp_dir().join("fp-cache-missing-nowhere");
        assert_eq!(clear(&e), Cleared { freed: 0, pending: false });
    }

    #[test]
    fn locked_file_leaves_a_marker_that_the_next_start_finishes() {
        let e = engine("locked");
        let locked = e.join("EBWebView/Default/Cache/Cache_Data/f_0001");
        // Windows не даёт удалить файл, открытый без FILE_SHARE_DELETE — как у работающего WebView2
        let handle = {
            use std::os::windows::fs::OpenOptionsExt;
            fs::OpenOptions::new().read(true).share_mode(0).open(&locked).unwrap()
        };
        let r = clear(&e);
        assert!(r.pending);
        assert!(e.join(PENDING).exists());
        drop(handle);
        finish_pending(&e);
        assert_eq!(size(&e), 0);
        assert!(!e.join(PENDING).exists());
        let _ = fs::remove_dir_all(e);
    }
}
