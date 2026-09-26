use crate::gpu::GpuInfo;
use crate::model::{Notice, Server, ServerStats, Settings};
use crate::store::Store;
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct Data {
    pub settings: Settings,
    pub servers: Vec<Server>,
    pub stats: BTreeMap<String, ServerStats>,
    pub notice: Option<Notice>,
    /// Сервер, открытый в окне `game` (для атрибуции телеметрии).
    pub current_server: Option<String>,
    /// Откат ANGLE выполняется не больше одного раза за сессию.
    pub session_fallback_done: bool,
}

pub struct AppState {
    pub store: Store,
    pub gpu: Option<GpuInfo>,
    pub data: Mutex<Data>,
}
