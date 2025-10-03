use wgpu::*;

#[derive(Debug)]
pub struct CommonCanvasRenderer {
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}

impl CommonCanvasRenderer {
    pub fn new(device: Device, queue: Queue, shader: &str) -> Self {
        
        let pipeline = Self::create_render_pipeline(&device, shader);

        Self {
            device,
            queue,
            render_pipeline: pipeline,
        }
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

    pub fn set_shader(&mut self, shader: &str) {
        self.render_pipeline = Self::create_render_pipeline(&self.device, shader);
    }

    fn create_render_pipeline(device: &Device, shader: &str) -> RenderPipeline {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(shader)),
        });

        let pipeline_layout = device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        device
            .create_render_pipeline(&RenderPipelineDescriptor {
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
