//! Explicit desktop backend selection; unsupported choices fail instead of falling back silently.
use winit::event_loop::EventLoop;

/// A preference is a hint; an adapter name is a strict, unambiguous selection.
#[derive(Clone, Debug)]
pub struct GpuSelection {
    pub power_preference: wgpu::PowerPreference,
    pub adapter_name: Option<String>,
}

impl Default for GpuSelection {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            adapter_name: None,
        }
    }
}

impl GpuSelection {
    pub fn parse_power_preference(value: &str) -> Result<wgpu::PowerPreference, String> {
        match value {
            "high-performance" => Ok(wgpu::PowerPreference::HighPerformance),
            "low-power" => Ok(wgpu::PowerPreference::LowPower),
            "none" => Ok(wgpu::PowerPreference::None),
            _ => Err(format!(
                "invalid power preference {value:?}; use high-performance, low-power, or none"
            )),
        }
    }

    pub async fn request_adapter(
        &self,
        instance: &wgpu::Instance,
        backends: wgpu::Backends,
        surface: &wgpu::Surface<'_>,
    ) -> Result<wgpu::Adapter, String> {
        if let Some(requested) = &self.adapter_name {
            let mut adapters: Vec<_> = instance
                .enumerate_adapters(backends)
                .await
                .into_iter()
                .filter(|adapter| adapter.is_surface_supported(surface))
                .collect();
            let names: Vec<_> = adapters
                .iter()
                .map(|adapter| adapter.get_info().name)
                .collect();
            let index = unique_adapter_index(&names, requested)?;
            return Ok(adapters.remove(index));
        }
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: self.power_preference,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await
            .map_err(|error| format!("could not find a compatible {backends:?} GPU adapter: {error}; check the GPU driver or select an explicit supported --backend"))
    }
}

fn unique_adapter_index(names: &[String], requested: &str) -> Result<usize, String> {
    let needle = requested.trim().to_lowercase();
    if needle.is_empty() {
        return Err("--adapter requires a non-empty GPU name or substring".to_owned());
    }
    let matches: Vec<_> = names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.to_lowercase().contains(&needle))
        .map(|(index, _)| index)
        .collect();
    match matches.as_slice() {
        [index] => Ok(*index),
        _ => Err(format!(
            "--adapter {requested:?} matched {} presentation-compatible GPUs; expected exactly one. Available on the selected backend: {names:?}",
            matches.len()
        )),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GpuBackend {
    #[default]
    Auto,
    Metal,
    Dx12,
    Vulkan,
    Gl,
}

impl GpuBackend {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "metal" => Ok(Self::Metal),
            "dx12" => Ok(Self::Dx12),
            "vulkan" => Ok(Self::Vulkan),
            "gl" => Ok(Self::Gl),
            _ => Err(format!(
                "invalid backend {value:?}; use auto, metal, dx12, vulkan, or gl"
            )),
        }
    }

    pub fn mask(self, os: &str) -> Result<wgpu::Backends, String> {
        use wgpu::Backends as B;
        match (os, self) {
            ("macos", Self::Auto | Self::Metal) => Ok(B::METAL),
            ("windows", Self::Auto | Self::Dx12) => Ok(B::DX12),
            ("linux", Self::Auto | Self::Vulkan) | ("windows", Self::Vulkan) => Ok(B::VULKAN),
            ("linux", Self::Gl) => Ok(B::GL),
            _ => Err(format!(
                "GPU backend {self:?} is not supported by SCShader on {os}"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowSystem {
    #[default]
    Auto,
    X11,
    Wayland,
}

impl WindowSystem {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "auto" => Ok(Self::Auto),
            "x11" => Ok(Self::X11),
            "wayland" => Ok(Self::Wayland),
            _ => Err(format!(
                "invalid window system {value:?}; use auto, x11, or wayland"
            )),
        }
    }

    pub fn validate(self, os: &str) -> Result<(), String> {
        if self != Self::Auto && os != "linux" {
            return Err("explicit X11/Wayland selection is supported only on Linux".to_owned());
        }
        Ok(())
    }

    pub fn event_loop(self) -> Result<EventLoop<()>, String> {
        self.validate(std::env::consts::OS)?;
        let mut builder = EventLoop::builder();
        #[cfg(target_os = "linux")]
        match self {
            Self::Auto => {}
            Self::X11 => {
                use winit::platform::x11::EventLoopBuilderExtX11;
                builder.with_x11();
            }
            Self::Wayland => {
                use winit::platform::wayland::EventLoopBuilderExtWayland;
                builder.with_wayland();
            }
        }
        builder.build().map_err(|error| {
            format!(
                "could not open {self:?} display: {error}; run in an interactive desktop session"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_gpu_selection_is_explicit_and_case_insensitive() {
        let names = vec!["Intel UHD Graphics 630".into(), "NVIDIA GTX 1050 Ti".into()];
        assert_eq!(unique_adapter_index(&names, "nvidia"), Ok(1));
        assert_eq!(unique_adapter_index(&names, " Intel "), Ok(0));
        assert!(unique_adapter_index(&names, "AMD").is_err());
        assert!(unique_adapter_index(&names, " ").is_err());
        assert!(unique_adapter_index(&[], "NVIDIA").is_err());
        let duplicates = vec!["NVIDIA GPU 1".into(), "NVIDIA GPU 2".into()];
        assert!(unique_adapter_index(&duplicates, "NVIDIA").is_err());
    }

    #[test]
    fn power_preference_preserves_default_and_rejects_typos() {
        assert_eq!(
            GpuSelection::default().power_preference,
            wgpu::PowerPreference::HighPerformance
        );
        assert_eq!(
            GpuSelection::parse_power_preference("low-power"),
            Ok(wgpu::PowerPreference::LowPower)
        );
        assert_eq!(
            GpuSelection::parse_power_preference("none"),
            Ok(wgpu::PowerPreference::None)
        );
        assert!(GpuSelection::parse_power_preference("intel").is_err());
    }

    #[test]
    fn desktop_defaults_are_explicit() {
        assert_eq!(GpuBackend::Auto.mask("macos"), Ok(wgpu::Backends::METAL));
        assert_eq!(GpuBackend::Auto.mask("windows"), Ok(wgpu::Backends::DX12));
        assert_eq!(GpuBackend::Auto.mask("linux"), Ok(wgpu::Backends::VULKAN));
    }

    #[test]
    fn unsupported_or_unknown_backends_fail() {
        assert!(GpuBackend::parse("typo").is_err());
        assert!(GpuBackend::Metal.mask("windows").is_err());
        assert!(GpuBackend::Dx12.mask("linux").is_err());
        assert!(GpuBackend::Auto.mask("unknown").is_err());
        assert_eq!(
            GpuBackend::parse("vulkan").unwrap().mask("windows"),
            Ok(wgpu::Backends::VULKAN)
        );
        assert_eq!(GpuBackend::Gl.mask("linux"), Ok(wgpu::Backends::GL));
    }

    #[test]
    fn display_overrides_are_linux_only() {
        for name in ["x11", "wayland"] {
            let system = WindowSystem::parse(name).unwrap();
            assert!(system.validate("linux").is_ok());
            assert!(system.validate("macos").is_err());
            assert!(system.validate("windows").is_err());
        }
        assert!(WindowSystem::parse("typo").is_err());
        assert!(WindowSystem::Auto.validate("windows").is_ok());
    }
}
