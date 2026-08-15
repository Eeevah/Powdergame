//! Minimal presentation layer for the G0 Windows app.
//!
//! The Renderer owns the surface + clear/present path only. It is NOT the
//! authoritative owner of simulation state
//! (`docs/architecture/ARCHITECTURE.md` §15, MILESTONES G0).

use std::sync::Arc;

use wgpu::TextureFormat;

use powdergame_gpu::GpuError;
use winit::window::Window;

/// Window surface renderer: acquire → clear → present.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// Clear color for the empty G0 world frame (a dim slate blue).
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.02,
    g: 0.02,
    b: 0.05,
    a: 1.0,
};

impl Renderer {
    /// Creates a surface for `window` on the given instance/adapter/device.
    pub fn new(
        instance: &wgpu::Instance,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: Arc<Window>,
    ) -> Result<Self, GpuError> {
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| GpuError::SurfaceCreateFailed(e.to_string()))?;

        let capabilities = surface.get_capabilities(adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(device, &config);

        Ok(Self {
            surface,
            config,
            device: device.clone(),
            queue: queue.clone(),
        })
    }

    /// Reconfigures the surface after a window resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    /// Acquires a frame, clears it, and presents it.
    pub fn render(&mut self) -> Result<(), GpuError> {
        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| GpuError::SurfaceFrameAcquireFailed(e.to_string()))?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("powdergame-render-encoder"),
            });
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("powdergame-clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            // In wgpu 26 the render pass ends implicitly when dropped.
            drop(render_pass);
        }

        self.queue.submit([encoder.finish()]);
        frame.present();
        Ok(())
    }

    /// The surface format in use (useful for diagnostics).
    pub fn format(&self) -> TextureFormat {
        self.config.format
    }
}
