//! Где окна были в прошлый раз: лаунчер, основное окно игры и окно мира,
//! которое игра открывает сама (Sqyre). Координаты — физические пиксели Windows.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Сколько окна должно остаться на каком-нибудь мониторе, чтобы за него можно было взяться.
const MIN_VISIBLE_W: i64 = 120;
const MIN_VISIBLE_H: i64 = 60;
/// Больше любого реального монитора — такой размер мог прийти только из испорченного файла.
const MAX_SIDE: u32 = 16_384;
/// Размер «обычного» окна, если его видели только развёрнутым.
const FALLBACK_W: u32 = 1280;
const FALLBACK_H: u32 = 800;
/// Развёрнутое окно Windows вылезает за край монитора на ширину рамки — сдвигаем внутрь.
const MAXIMIZED_INSET: i32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Launcher,
    Game,
    /// Первое окно без заданного размера, открытое игрой (у Sqyre — сам мир).
    Popup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    fn overlap(&self, o: &Rect) -> (i64, i64) {
        let w = (self.x as i64 + self.w as i64).min(o.x as i64 + o.w as i64) - (self.x as i64).max(o.x as i64);
        let h = (self.y as i64 + self.h as i64).min(o.y as i64 + o.h as i64) - (self.y as i64).max(o.y as i64);
        (w.max(0), h.max(0))
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && y >= self.y && (x as i64) < self.x as i64 + self.w as i64 && (y as i64) < self.y as i64 + self.h as i64
    }
}

/// «Обычный» прямоугольник окна плюс поверх него — развёрнуто ли и на весь ли экран.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub maximized: bool,
    pub fullscreen: bool,
}

impl Placement {
    pub fn rect(&self) -> Rect {
        Rect { x: self.x, y: self.y, w: self.w, h: self.h }
    }
}

/// Текущее состояние окна, как его отдаёт Tauri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Snapshot {
    pub rect: Rect,
    pub maximized: bool,
    pub fullscreen: bool,
    pub minimized: bool,
}

/// Окно открываем на прежнем месте, только если оно заметно видно хотя бы на одном мониторе.
pub fn fits(p: &Placement, monitors: &[Rect]) -> bool {
    (1..=MAX_SIDE).contains(&p.w) && (1..=MAX_SIDE).contains(&p.h) && monitors.iter().any(|m| {
        let (w, h) = p.rect().overlap(m);
        w >= MIN_VISIBLE_W && h >= MIN_VISIBLE_H
    })
}

fn monitor_of(x: i32, y: i32, monitors: &[Rect]) -> Option<usize> {
    monitors.iter().position(|m| m.contains(x, y))
}

/// Новое запоминаемое положение после движения или смены размера окна.
/// Свёрнутое окно ничего не меняет; развёрнутое хранит прежний обычный прямоугольник,
/// но переезжает на тот монитор, где его развернули.
pub fn merge(prev: Option<Placement>, s: Snapshot, monitors: &[Rect]) -> Option<Placement> {
    if s.minimized {
        return prev;
    }
    if !s.maximized && !s.fullscreen {
        return Some(Placement { x: s.rect.x, y: s.rect.y, w: s.rect.w, h: s.rect.h, maximized: false, fullscreen: false });
    }
    let (inside_x, inside_y) = (s.rect.x + MAXIMIZED_INSET, s.rect.y + MAXIMIZED_INSET);
    let base = match prev {
        Some(p) if monitor_of(p.x + MAXIMIZED_INSET, p.y + MAXIMIZED_INSET, monitors) == monitor_of(inside_x, inside_y, monitors) => p,
        Some(p) => Placement { x: inside_x, y: inside_y, ..p },
        None => Placement { x: inside_x, y: inside_y, w: FALLBACK_W, h: FALLBACK_H, maximized: false, fullscreen: false },
    };
    Some(Placement { maximized: s.maximized, fullscreen: s.fullscreen, ..base })
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Saved(pub BTreeMap<Role, Placement>);

pub fn file_path(dir: &Path) -> PathBuf {
    dir.join("windows.json")
}

/// Нет файла или он битый — просто открываем окна как в первый раз.
pub fn load(dir: &Path) -> Saved {
    fs::read_to_string(file_path(dir)).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn save(dir: &Path, saved: &Saved) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join("windows.json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(saved)?)?;
    fs::rename(tmp, file_path(dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMARY: Rect = Rect { x: 0, y: 0, w: 1920, h: 1080 };
    const SECOND: Rect = Rect { x: 1920, y: 0, w: 2560, h: 1440 };

    fn normal(x: i32, y: i32, w: u32, h: u32) -> Snapshot {
        Snapshot { rect: Rect { x, y, w, h }, maximized: false, fullscreen: false, minimized: false }
    }

    fn placed(x: i32, y: i32) -> Placement {
        Placement { x, y, w: 1000, h: 700, maximized: false, fullscreen: false }
    }

    #[test]
    fn window_on_a_connected_monitor_fits() {
        assert!(fits(&placed(100, 100), &[PRIMARY]));
        assert!(fits(&placed(2500, 300), &[PRIMARY, SECOND]));
    }

    #[test]
    fn window_on_a_disconnected_monitor_does_not_fit() {
        assert!(!fits(&placed(2500, 300), &[PRIMARY]));
    }

    #[test]
    fn a_sliver_at_the_edge_is_not_enough() {
        // видно 50 px по ширине — за заголовок не ухватиться
        assert!(!fits(&placed(1870, 100), &[PRIMARY]));
        assert!(fits(&placed(1700, 100), &[PRIMARY]));
    }

    #[test]
    fn zero_or_absurd_size_never_fits() {
        assert!(!fits(&Placement { w: 0, ..placed(10, 10) }, &[PRIMARY]));
        assert!(!fits(&Placement { h: 65_496, ..placed(10, 10) }, &[PRIMARY]));
    }

    #[test]
    fn normal_window_is_remembered_as_is() {
        let p = merge(None, normal(40, 50, 1200, 800), &[PRIMARY]).unwrap();
        assert_eq!(p, Placement { x: 40, y: 50, w: 1200, h: 800, maximized: false, fullscreen: false });
    }

    #[test]
    fn minimizing_keeps_the_previous_placement() {
        let prev = Some(placed(10, 10));
        let s = Snapshot { rect: Rect { x: -32000, y: -32000, w: 160, h: 28 }, minimized: true, ..normal(0, 0, 0, 0) };
        assert_eq!(merge(prev, s, &[PRIMARY]), prev);
    }

    #[test]
    fn maximizing_keeps_the_normal_rect() {
        let s = Snapshot { maximized: true, ..normal(-8, -8, 1936, 1056) };
        let p = merge(Some(placed(200, 150)), s, &[PRIMARY]).unwrap();
        assert_eq!((p.x, p.y, p.w, p.maximized), (200, 150, 1000, true));
    }

    #[test]
    fn maximized_on_another_monitor_moves_there() {
        let s = Snapshot { maximized: true, ..normal(1912, -8, 2576, 1416) };
        let p = merge(Some(placed(200, 150)), s, &[PRIMARY, SECOND]).unwrap();
        assert!(SECOND.contains(p.x, p.y), "{p:?}");
        assert_eq!((p.w, p.h, p.maximized), (1000, 700, true));
    }

    #[test]
    fn first_seen_maximized_gets_a_fallback_size_on_that_monitor() {
        let s = Snapshot { maximized: true, ..normal(1912, -8, 2576, 1416) };
        let p = merge(None, s, &[PRIMARY, SECOND]).unwrap();
        assert!(SECOND.contains(p.x, p.y));
        assert_eq!((p.w, p.h), (FALLBACK_W, FALLBACK_H));
        assert!(fits(&p, &[PRIMARY, SECOND]));
    }

    #[test]
    fn fullscreen_is_remembered() {
        let s = Snapshot { fullscreen: true, ..normal(0, 0, 1920, 1080) };
        assert!(merge(Some(placed(10, 10)), s, &[PRIMARY]).unwrap().fullscreen);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("fp-winstate-{}", std::process::id()));
        let mut saved = Saved::default();
        saved.0.insert(Role::Popup, placed(5, 6));
        save(&dir, &saved).unwrap();
        assert_eq!(load(&dir), saved);
        fs::write(file_path(&dir), "{broken").unwrap();
        assert_eq!(load(&dir), Saved::default());
        let _ = fs::remove_dir_all(dir);
    }
}
