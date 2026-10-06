use crate::model::{AngleBackend, ProfileId, Verdict};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub vram_mb: u64,
    pub vendor_id: u32,
}

const VENDOR_NVIDIA: u32 = 0x10DE;
const VENDOR_AMD: u32 = 0x1002;
const VENDOR_INTEL: u32 = 0x8086;
const VENDOR_MICROSOFT_BASIC: u32 = 0x1414;
/// Признаки программного рендера в строке WebGL (в нижнем регистре).
const SOFTWARE_MARKERS: [&str; 3] = ["swiftshader", "basic render driver", "llvmpipe"];

pub fn recommend(gpu: Option<&GpuInfo>) -> ProfileId {
    match gpu {
        None => ProfileId::Balance,
        Some(g) if g.vendor_id == VENDOR_INTEL => ProfileId::Potato,
        Some(g) if g.vram_mb >= 8 * 1024 => ProfileId::Quality,
        Some(g) if g.vram_mb >= 4 * 1024 => ProfileId::Balance,
        Some(_) => ProfileId::Potato,
    }
}

/// Все аппаратные адаптеры, без программных.
pub fn detect_all() -> Vec<GpuInfo> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE};

    let mut out = Vec::new();
    unsafe {
        let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() else { return out };
        let mut i = 0u32;
        while let Ok(adapter) = factory.EnumAdapters1(i) {
            i += 1;
            let Ok(desc) = adapter.GetDesc1() else { continue };
            if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 || desc.VendorId == VENDOR_MICROSOFT_BASIC {
                continue;
            }
            let len = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
            out.push(GpuInfo {
                name: String::from_utf16_lossy(&desc.Description[..len]),
                vram_mb: (desc.DedicatedVideoMemory as u64) / (1024 * 1024),
                vendor_id: desc.VendorId,
            });
        }
    }
    out
}

/// Адаптер с наибольшей выделенной VRAM; при равенстве — первый.
pub fn best(adapters: &[GpuInfo]) -> Option<&GpuInfo> {
    adapters.iter().fold(None, |b: Option<&GpuInfo>, a| if b.is_none_or(|b| a.vram_mb > b.vram_mb) { Some(a) } else { b })
}

/// Производитель из первого поля строки ANGLE: `ANGLE (NVIDIA, …)`.
fn renderer_vendor(renderer: &str) -> Option<u32> {
    let field = renderer.strip_prefix("ANGLE (")?.split(',').next()?.trim().to_ascii_lowercase();
    if field.starts_with("nvidia") {
        Some(VENDOR_NVIDIA)
    } else if field.starts_with("amd") || field.starts_with("ati ") || field.starts_with("advanced micro devices") {
        Some(VENDOR_AMD)
    } else if field.starts_with("intel") {
        Some(VENDOR_INTEL)
    } else {
        None
    }
}

/// Бэкенд, на котором ANGLE рисует на самом деле; `lower` — строка в нижнем регистре.
fn renderer_backend(lower: &str) -> Option<AngleBackend> {
    if lower.contains("on12") {
        Some(AngleBackend::D3d11on12)
    } else if lower.contains("direct3d11") || lower.contains("d3d11") {
        Some(AngleBackend::D3d11)
    } else if lower.contains("vulkan") {
        Some(AngleBackend::Vulkan)
    } else if lower.contains("opengl") {
        Some(AngleBackend::Gl)
    } else {
        None
    }
}

/// На чём игра рисует: по строке `UNMASKED_RENDERER_WEBGL` и адаптерам DXGI.
pub fn classify(renderer: &str, adapters: &[GpuInfo]) -> Verdict {
    let lower = renderer.to_ascii_lowercase();
    if SOFTWARE_MARKERS.iter().any(|m| lower.contains(m)) {
        return Verdict::Software;
    }
    let Some(vendor) = renderer_vendor(renderer) else { return Verdict::Unknown };
    let backend = renderer_backend(&lower);
    // Встройка при живой дискретной: производитель есть среди адаптеров, но это не лучший
    let wrong = best(adapters).is_some_and(|b| b.vendor_id != vendor) && adapters.iter().any(|a| a.vendor_id == vendor);
    if wrong {
        Verdict::WrongGpu { backend }
    } else {
        Verdict::Hardware { backend }
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
        println!("{:?}", detect_all());
    }

    const NV_D3D11: &str = "ANGLE (NVIDIA, NVIDIA GeForce GTX 1060 6GB (0x00001C03) Direct3D11 vs_5_0 ps_5_0, D3D11)";
    const NV_ON12: &str = "ANGLE (NVIDIA, NVIDIA GeForce RTX 5070 (0x00002F04) Direct3D11on12 vs_5_0 ps_5_0, D3D11on12)";
    const NV_GL: &str = "ANGLE (NVIDIA Corporation, NVIDIA GeForce GTX 1060 6GB/PCIe/SSE2, OpenGL 4.5.0)";
    const NV_VULKAN: &str = "ANGLE (NVIDIA, Vulkan 1.3.277 (NVIDIA GeForce RTX 5070 (0x00002F04)), NVIDIA)";
    const AMD_GL: &str = "ANGLE (ATI Technologies Inc., AMD Radeon RX 580 Series, OpenGL 4.5.0)";
    const INTEL_D3D11: &str = "ANGLE (Intel, Intel(R) UHD Graphics 620 (0x00005917) Direct3D11 vs_5_0 ps_5_0, D3D11)";
    const SWIFTSHADER: &str = "ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)";
    const BASIC: &str = "ANGLE (Microsoft, Microsoft Basic Render Driver (0x0000008C) Direct3D11 vs_5_0 ps_5_0, D3D11)";

    fn nv() -> GpuInfo {
        g(6144, 0x10DE)
    }
    fn intel() -> GpuInfo {
        g(128, 0x8086)
    }
    fn hw(b: AngleBackend) -> Verdict {
        Verdict::Hardware { backend: Some(b) }
    }

    #[test]
    fn best_prefers_vram_then_the_first() {
        let list = [intel(), nv(), g(6144, 0x1002)];
        assert_eq!(best(&list).unwrap().vendor_id, 0x10DE);
        assert_eq!(best(&[]), None);
    }

    #[test]
    fn hardware_reports_the_real_backend() {
        assert_eq!(classify(NV_D3D11, &[nv()]), hw(AngleBackend::D3d11));
        assert_eq!(classify(NV_ON12, &[nv()]), hw(AngleBackend::D3d11on12));
        assert_eq!(classify(NV_GL, &[nv()]), hw(AngleBackend::Gl));
        assert_eq!(classify(NV_VULKAN, &[nv()]), hw(AngleBackend::Vulkan));
        assert_eq!(classify(AMD_GL, &[g(8192, 0x1002)]), hw(AngleBackend::Gl));
    }

    #[test]
    fn integrated_gpu_on_a_hybrid_laptop_is_the_wrong_gpu() {
        let hybrid = [intel(), nv()];
        assert_eq!(classify(INTEL_D3D11, &hybrid), Verdict::WrongGpu { backend: Some(AngleBackend::D3d11) });
        assert_eq!(classify(NV_D3D11, &hybrid), hw(AngleBackend::D3d11));
    }

    #[test]
    fn a_lone_intel_or_unknown_adapters_are_hardware() {
        assert_eq!(classify(INTEL_D3D11, &[intel()]), hw(AngleBackend::D3d11));
        assert_eq!(classify(INTEL_D3D11, &[]), hw(AngleBackend::D3d11));
    }

    #[test]
    fn software_renderers_are_detected() {
        assert_eq!(classify(SWIFTSHADER, &[nv()]), Verdict::Software);
        assert_eq!(classify(BASIC, &[nv()]), Verdict::Software);
    }

    #[test]
    fn unrecognized_strings_are_unknown() {
        for s in ["", "garbage", "ANGLE (Qualcomm, Adreno 690)", "WebKit WebGL"] {
            assert_eq!(classify(s, &[nv()]), Verdict::Unknown, "{s}");
        }
    }
}
