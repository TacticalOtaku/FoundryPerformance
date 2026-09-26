use crate::model::ProfileId;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub vendor_id: u32,
}

const VENDOR_INTEL: u32 = 0x8086;
const VENDOR_MICROSOFT_BASIC: u32 = 0x1414;

pub fn recommend(gpu: Option<&GpuInfo>) -> ProfileId {
    match gpu {
        None => ProfileId::Balance,
        Some(g) if g.vendor_id == VENDOR_INTEL => ProfileId::Potato,
        Some(g) if g.vram_mb >= 8 * 1024 => ProfileId::Quality,
        Some(g) if g.vram_mb >= 4 * 1024 => ProfileId::Balance,
        Some(_) => ProfileId::Potato,
    }
}

/// Адаптер с наибольшей выделенной VRAM, без программных.
pub fn detect() -> Option<GpuInfo> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};

    unsafe {
        let factory: IDXGIFactory1 = CreateDXGIFactory1().ok()?;
        let mut best: Option<GpuInfo> = None;
        let mut i = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(i) {
            i += 1;
            let Ok(desc) = adapter.GetDesc1() else { continue };
            if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 || desc.VendorId == VENDOR_MICROSOFT_BASIC {
                continue;
            }
            let len = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            let info = GpuInfo {
                name: String::from_utf16_lossy(&desc.Description[..len]),
                vram_mb: (desc.DedicatedVideoMemory as u64) / (1024 * 1024),
                vendor_id: desc.VendorId,
            };
            if best.as_ref().is_none_or(|b| info.vram_mb > b.vram_mb) {
                best = Some(info);
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(vram_mb: u64, vendor_id: u32) -> GpuInfo {
        GpuInfo { name: "x".into(), vram_mb, vendor_id }
    }

    #[test]
    fn recommendation_by_vram_and_vendor() {
        assert_eq!(recommend(Some(&g(12 * 1024, 0x10DE))), ProfileId::Quality);
        assert_eq!(recommend(Some(&g(6 * 1024, 0x10DE))), ProfileId::Balance);
        assert_eq!(recommend(Some(&g(3 * 1024, 0x10DE))), ProfileId::Potato);
        assert_eq!(recommend(Some(&g(8 * 1024, 0x8086))), ProfileId::Potato);
        assert_eq!(recommend(None), ProfileId::Balance);
    }

    #[test]
    #[ignore]
    fn print_detected_gpu() {
        println!("{:?}", detect());
    }
}
