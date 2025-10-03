use wgpu::*;

use crate::wgpu_render::common::CommonCanvasRenderer;

/// Canvas renderer that works on a canvas via the web
pub struct WebCanvasRenderer {
    surface: Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    canvas_renderer: CommonCanvasRenderer,
    is_surface_configured: bool,
}

impl WebCanvasRenderer {
    pub async fn new(canvas_element: web_sys::HtmlCanvasElement, shader: &str) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface_target = wgpu::SurfaceTarget::Canvas(canvas_element);
        let surface = instance
            .create_surface(surface_target)
            .expect("Can't create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(WebCanvasError::AdapterFailed)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|_| WebCanvasError::DeviceFailed)?;

        let surface_capabilities = surface.get_capabilities(&adapter);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Rgba8Unorm,
            width: 500,
            height: 500,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let canvas_renderer = CommonCanvasRenderer::new(device, queue, shader);

        Ok(Self {
            surface,
            config,
            is_surface_configured: false,
            canvas_renderer,
        })
    }

    pub fn render(&mut self) {
        if !self.is_surface_configured {
            self.surface
                .configure(&self.canvas_renderer.device(), &self.config);
            self.is_surface_configured = true;
        }

        let output = self.surface.get_current_texture().unwrap();

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.canvas_renderer.render(view);

        output.present();
    }

    pub fn set_shader(&mut self, shader: &str) {
        self.canvas_renderer.set_shader(shader);
    }
}

#[derive(Debug)]
enum WebCanvasError {
    AdapterFailed,
    DeviceFailed,
}

impl std::fmt::Display for WebCanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AdapterFailed => f.write_str("failed requesting adapter"),
            Self::DeviceFailed => f.write_str("failed to request device"),
        }
    }
}

impl std::error::Error for WebCanvasError {}
