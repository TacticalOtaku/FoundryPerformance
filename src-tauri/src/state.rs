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
    /// Источники страниц, от которых в этой сессии принимаются отчёты агента:
    /// адрес входа, известный адрес игры и адрес, подтверждённый проверкой Foundry.
    pub session_origins: Vec<url::Origin>,
}

pub struct AppState {
    pub store: Store,
    /// Аппаратные адаптеры DXGI; паспорт и рекомендация берут `gpu::best`.
    pub adapters: Vec<GpuInfo>,
    pub data: Mutex<Data>,
}
