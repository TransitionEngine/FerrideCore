use super::ShaderDescriptor;
use std::fmt::Debug;

use super::RenderScene;

pub trait WindowSurface: Debug {
    fn surface<'a, 'b: 'a>(&'b self) -> &'a wgpu::Surface<'a>;
    fn config(&self) -> &wgpu::SurfaceConfiguration;
    fn config_mut(&mut self) -> &mut wgpu::SurfaceConfiguration;
    fn resize(&mut self, new_size: &winit::dpi::PhysicalSize<u32>, device: &wgpu::Device) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.config_mut().width = new_size.width;
        self.config_mut().height = new_size.height;
        self.surface().configure(device, self.config());
    }
    fn create_render_pipeline<'a>(
        &self,
        device: &wgpu::Device,
        bind_group_layout: &[&wgpu::BindGroupLayout],
        shader: &wgpu::ShaderModule,
        shader_descriptor: &ShaderDescriptor,
        vertex_buffer_layout: wgpu::VertexBufferLayout<'a>,
    ) -> wgpu::RenderPipeline;
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_scenes: &[&RenderScene],
        texture_bind_group: &wgpu::BindGroup,
    );
}

pub struct Surface<'a> {
    pub wgpu_surface: wgpu::Surface<'a>,
    pub config: wgpu::SurfaceConfiguration,
}
impl Debug for Surface<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("config", &self.config)
            .finish()
    }
}
impl<'a> WindowSurface for Surface<'a> {
    fn surface<'b, 'c: 'b>(&'c self) -> &'b wgpu::Surface<'b> {
        &self.wgpu_surface
    }

    fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    fn config_mut(&mut self) -> &mut wgpu::SurfaceConfiguration {
        &mut self.config
    }

    fn create_render_pipeline<'b>(
        &self,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        shader: &wgpu::ShaderModule,
        shader_descriptor: &ShaderDescriptor,
        vertex_buffer_layout: wgpu::VertexBufferLayout<'b>,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: shader_descriptor.vertex_shader,
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: shader_descriptor.fragment_shader,
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        render_pipeline
    }

    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_scenes: &[&RenderScene],
        texture_bind_group: &wgpu::BindGroup,
    ) {
        let output = self
            .surface()
            .get_current_texture()
            .expect("Our food has no texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for render_scene in render_scenes {
                render_scene.write_render_pass(&mut render_pass, texture_bind_group);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
