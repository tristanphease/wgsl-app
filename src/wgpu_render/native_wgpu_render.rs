//! Based on https://github.com/DioxusLabs/dioxus/blob/539ede20416109bb378b87f3458f7dccfa065f61/examples/wgpu-texture/src/demo_renderer.rs

use std::sync::mpsc::{channel, Receiver, Sender};

use dioxus_native::{CustomPaintCtx, CustomPaintSource, TextureHandle};
use wgpu::*;

use crate::wgpu_render::common::CommonCanvasRenderer;

enum CanvasRendererState {
    Active(Box<ActiveCanvasRenderer>),
    Suspended,
}

pub struct CanvasPaintSource {
    state: CanvasRendererState,
    tx: Sender<CanvasMessage>,
    rx: Receiver<CanvasMessage>,
    current_shader: String,
}

impl CustomPaintSource for CanvasPaintSource {
    fn resume(&mut self, _instance: &Instance, device_handle: &dioxus_native::DeviceHandle) {
        // Extract device and queue from device handle
        let device = &device_handle.device;
        let queue = &device_handle.queue;
        let active_state = ActiveCanvasRenderer::new(device, queue, &self.current_shader);
        self.state = CanvasRendererState::Active(Box::new(active_state));
    }

    fn suspend(&mut self) {
        self.state = CanvasRendererState::Suspended;
    }

    fn render(
        &mut self,
        ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
        _scale: f64,
    ) -> Option<TextureHandle> {
        self.process_messages();
        self.render(ctx, width, height)
    }
}

impl CanvasPaintSource {
    pub fn new(shader: String) -> Self {
        let (tx, rx) = channel();
        Self::with_channel(tx, rx, shader)
    }

    pub fn with_channel(tx: Sender<CanvasMessage>, rx: Receiver<CanvasMessage>, shader: String) -> Self {
        Self {
            state: CanvasRendererState::Suspended,
            tx,
            rx,
            current_shader: shader,
        }
    }

    pub fn sender(&self) -> Sender<CanvasMessage> {
        self.tx.clone()
    }

    fn process_messages(&mut self) {
        loop {
            match self.rx.try_recv() {
                Err(_) => return,
                Ok(msg) => match msg {
                    CanvasMessage::SetShader(new_shader) => {
                        if let CanvasRendererState::Active(ref mut renderer) = self.state {
                            renderer.common_renderer.set_shader(&new_shader);
                        }
                    }
                },
            }
        }
    }

    fn render(
        &mut self,
        ctx: dioxus_native::CustomPaintCtx<'_>,
        width: u32,
        height: u32,
    ) -> Option<dioxus_native::TextureHandle> {
        if width == 0 || height == 0 {
            return None;
        }
        let CanvasRendererState::Active(state) = &mut self.state else {
            return None;
        };

        state.render(ctx, width, height)
    }
}

pub enum CanvasMessage {
    SetShader(String),
}

#[derive(Clone)]
struct TextureAndHandle {
    texture: Texture,
    handle: TextureHandle,
}

struct ActiveCanvasRenderer {
    device: Device,
    common_renderer: CommonCanvasRenderer,
    displayed_texture: Option<TextureAndHandle>,
    next_texture: Option<TextureAndHandle>,
}

impl ActiveCanvasRenderer {
    fn new(device: &Device, queue: &Queue, shader: &str) -> Self {
        let common_renderer = CommonCanvasRenderer::new(device.clone(), queue.clone(), shader);

        Self {
            device: device.clone(),
            common_renderer,
            displayed_texture: None,
            next_texture: None,
        }
    }

    fn render(
        &mut self,
        mut ctx: CustomPaintCtx<'_>,
        width: u32,
        height: u32,
    ) -> Option<TextureHandle> {
        // if next texture size doesn't match specified size just drop texture
        if let Some(next) = &self.next_texture {
            if next.texture.width() != width || next.texture.height() != height {
                ctx.unregister_texture(next.handle);
                self.next_texture = None;
            }
        }

        // if there is no next texture then create one and register it
        let texture_and_handle = match &self.next_texture {
            Some(next) => next,
            None => {
                let texture = create_texture(&self.device, width, height);
                let handle = ctx.register_texture(texture.clone());
                self.next_texture = Some(TextureAndHandle { texture, handle });
                self.next_texture.as_ref().unwrap()
            }
        };

        let next_texture = &texture_and_handle.texture;
        let next_texture_handle = texture_and_handle.handle;

        let view = next_texture.create_view(&TextureViewDescriptor::default());

        self.common_renderer.render(view);

        std::mem::swap(&mut self.next_texture, &mut self.displayed_texture);
        Some(next_texture_handle)
    }
}

fn create_texture(device: &Device, width: u32, height: u32) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}
