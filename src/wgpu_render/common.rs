use std::{error::Error, fmt::Display};

use wgpu::*;

#[derive(Debug)]
pub enum ShaderError {
    CompileError,
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaderError::CompileError => f.write_str("Couldn't compile shader"),
        }
    }
}

impl Error for ShaderError {}

#[derive(Debug)]
pub struct CommonCanvasRenderer {
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}

impl CommonCanvasRenderer {
    pub async fn new(device: Device, queue: Queue, shader: &str) -> Result<Self, ShaderError> {
        let shader = Self::create_shader_module(&device, shader).await?;
        let pipeline = Self::create_render_pipeline(&device, shader);

        Ok(Self {
            device,
            queue,
            render_pipeline: pipeline,
        })
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn render(&mut self, view: TextureView) {
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::GREEN),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..6, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }

    pub async fn set_shader(&mut self, shader: &str) -> Result<(), ShaderError> {
        let shader = Self::create_shader_module(&self.device, shader).await?;
        self.render_pipeline = Self::create_render_pipeline(&self.device, shader);
        Ok(())
    }

    pub async fn create_shader_module(
        device: &Device,
        shader: &str,
    ) -> Result<ShaderModule, ShaderError> {
        device.push_error_scope(ErrorFilter::Validation);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader)),
        });

        let error_future = device.pop_error_scope();
        // check for errors async
        let error = error_future.await;

        if let Some(_error) = error {
            return Err(ShaderError::CompileError);
        }

        Ok(shader)
    }

    fn create_render_pipeline(device: &Device, shader: ShaderModule) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(TextureFormat::Rgba8Unorm.into())],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }
}
