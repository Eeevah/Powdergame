//! GPU context initialization.
//!
//! G0 explicitly requests the DX12 backend with a high-performance adapter.
//! There is intentionally no broad fallback: if the DX12 path or the expected
//! hardware is not present, this fails loudly instead of silently degrading
//! (`docs/planning/MILESTONES.md` G0, `docs/development/DEVELOPMENT.md` §3).

use std::fmt::Write;

use wgpu::Backend;

/// Expected primary performance GPU vendor (NVIDIA).
pub const NVIDIA_VENDOR_ID: u32 = 0x10DE;
/// Expected primary performance GPU model name fragment.
pub const REFERENCE_GPU_NAME: &str = "RTX 5090";

/// A wgpu instance + DX12 adapter + device/queue.
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
}

/// Human-readable report of the selected adapter (evidence for G0).
#[derive(Debug, Clone)]
pub struct AdapterReport {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub device_type: String,
    pub backend: String,
    pub driver: String,
    pub driver_info: String,
}

impl AdapterReport {
    pub fn from_info(info: &wgpu::AdapterInfo) -> Self {
        Self {
            name: info.name.clone(),
            vendor: info.vendor,
            device: info.device,
            device_type: format!("{:?}", info.device_type),
            backend: format!("{:?}", info.backend),
            driver: info.driver.clone(),
            driver_info: info.driver_info.clone(),
        }
    }
}

impl std::fmt::Display for AdapterReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        let _ = writeln!(out, "Adapter name:  {}", self.name);
        let _ = writeln!(out, "Vendor:        0x{:04X}", self.vendor);
        let _ = writeln!(out, "Device:        0x{:04X}", self.device);
        let _ = writeln!(out, "Device type:   {}", self.device_type);
        let _ = writeln!(out, "Backend:       {}", self.backend);
        let _ = writeln!(out, "Driver:        {}", self.driver);
        let _ = writeln!(out, "Driver info:   {}", self.driver_info);
        f.write_str(out.trim_end())
    }
}

/// Renders an adapter info struct as a compact one-line description.
pub fn describe_adapter_info(info: &wgpu::AdapterInfo) -> String {
    format!(
        "{} (vendor=0x{:04X} device=0x{:04X} type={:?} backend={:?} driver={})",
        info.name, info.vendor, info.device, info.device_type, info.backend, info.driver_info
    )
}

/// Verifies the adapter is the expected production target.
///
/// G0 passes only when the backend is DX12 and the adapter is the NVIDIA
/// RTX 5090 reference target.
pub fn verify_target_hardware(info: &wgpu::AdapterInfo) -> Result<(), GpuError> {
    if info.backend != Backend::Dx12 {
        return Err(GpuError::UnexpectedBackend(info.backend));
    }
    let is_nvidia = info.vendor == NVIDIA_VENDOR_ID;
    let name_is_reference = info
        .name
        .to_ascii_lowercase()
        .contains(&REFERENCE_GPU_NAME.to_ascii_lowercase());
    if !(is_nvidia && name_is_reference) {
        return Err(GpuError::UnexpectedHardware {
            vendor: info.vendor,
            name: info.name.clone(),
        });
    }
    Ok(())
}

impl GpuContext {
    /// Initializes an instance, DX12 adapter, device and queue.
    pub async fn new() -> Result<Self, GpuError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .map_err(|_| GpuError::AdapterNotFound)?;

        let adapter_info = adapter.get_info();

        // G0: the backend must actually be DX12. No silent fallback.
        verify_target_hardware(&adapter_info)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("powdergame-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| GpuError::DeviceRequestFailed(e.to_string()))?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            adapter_info,
        })
    }
}

/// GPU context / world / simulation failure.
#[derive(Debug)]
pub enum GpuError {
    /// No adapter was found on the DX12 backend.
    AdapterNotFound,
    /// The selected adapter is not the DX12 backend (fallback was not used).
    UnexpectedBackend(Backend),
    /// The selected adapter is not the expected reference hardware.
    UnexpectedHardware { vendor: u32, name: String },
    /// Device creation failed.
    DeviceRequestFailed(String),
    /// Surface creation failed.
    SurfaceCreateFailed(String),
    /// Frame acquire failed.
    SurfaceFrameAcquireFailed(String),
    /// Compute pipeline creation failed.
    PipelineFailed(String),
    /// Shader compilation failed.
    ShaderCompileFailed(String),
    /// GPU readback (map) failed.
    ReadbackFailed(String),
    /// Buffer creation failed.
    BufferCreateFailed(String),
    /// A coordinate lies outside the finite world (Void). It must never be
    /// clamped into the domain or turned into a buffer index.
    CoordinateOutOfBounds { x: i64, y: i64 },
    /// A material value is neither `EMPTY` nor a registered Matter.
    InvalidMaterialValue(u32),
    /// Other error with a message.
    Other(String),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuError::AdapterNotFound => write!(f, "no DX12 adapter found"),
            GpuError::UnexpectedBackend(backend) => {
                write!(f, "expected DX12 backend but got {backend:?}")
            }
            GpuError::UnexpectedHardware { vendor, name } => write!(
                f,
                "expected NVIDIA RTX 5090 but got {name} (vendor=0x{vendor:04X})"
            ),
            GpuError::DeviceRequestFailed(msg) => write!(f, "device request failed: {msg}"),
            GpuError::SurfaceCreateFailed(msg) => write!(f, "surface create failed: {msg}"),
            GpuError::SurfaceFrameAcquireFailed(msg) => write!(f, "frame acquire failed: {msg}"),
            GpuError::PipelineFailed(msg) => write!(f, "pipeline creation failed: {msg}"),
            GpuError::ShaderCompileFailed(msg) => write!(f, "shader compilation failed: {msg}"),
            GpuError::ReadbackFailed(msg) => write!(f, "GPU readback failed: {msg}"),
            GpuError::BufferCreateFailed(msg) => write!(f, "buffer creation failed: {msg}"),
            GpuError::CoordinateOutOfBounds { x, y } => {
                write!(
                    f,
                    "coordinate ({x}, {y}) is outside the finite world (Void)"
                )
            }
            GpuError::InvalidMaterialValue(value) => {
                write!(
                    f,
                    "invalid material value {value}: must be EMPTY or a registered Matter"
                )
            }
            GpuError::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GpuError {}
