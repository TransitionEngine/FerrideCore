use wgpu::util::DeviceExt;

use crate::create_name_struct;

use super::{IndexBufferWriter, VertexBufferWriter, Visibility};

create_name_struct!(RenderSceneName);
create_name_struct!(UniformBufferName);

#[derive(Debug, Clone)]
pub struct RenderSceneDescriptor {
    pub index_format: wgpu::IndexFormat,
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    pub use_textures: bool,
}

pub struct RenderScene {
    name: RenderSceneName,
    render_pipeline: Option<wgpu::RenderPipeline>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    num_vertices: u32,
    index_format: wgpu::IndexFormat,
    vertex_buffer_layout: wgpu::VertexBufferLayout<'static>,
    use_textures: bool,
    uniform_buffers: Vec<(
        UniformBufferName,
        wgpu::Buffer,
        wgpu::BindGroupLayout,
        wgpu::BindGroup,
    )>,
    visibility: Visibility,
}
impl RenderScene {
    pub fn new(
        name: RenderSceneName,
        device: &wgpu::Device,
        descriptor: RenderSceneDescriptor,
    ) -> Self {
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Vertex Buffer {:?}", name)),
            size: 0,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Index Buffer {:?}", name)),
            size: 0,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let num_vertices = 0;
        let num_indices = 0;

        Self {
            name,
            render_pipeline: None,
            vertex_buffer,
            index_buffer,
            num_indices,
            num_vertices,
            index_format: descriptor.index_format,
            vertex_buffer_layout: descriptor.vertex_buffer_layout,
            use_textures: descriptor.use_textures,
            uniform_buffers: Vec::new(),
            visibility: Visibility::Visible,
        }
    }

    pub fn set_visibility(&mut self, visibility: &Visibility) {
        self.visibility = visibility.clone();
    }

    pub fn use_textures(&self) -> bool {
        self.use_textures
    }

    fn bind_groups<'a>(
        &'a self,
        texture_bind_group: &'a wgpu::BindGroup,
    ) -> Vec<&'a wgpu::BindGroup> {
        let mut bind_groups = if self.use_textures {
            vec![texture_bind_group]
        } else {
            Vec::new()
        };
        bind_groups.extend(self.uniform_buffers.iter().map(|(_, _, _, bg)| bg));
        bind_groups
    }

    pub fn bind_group_layouts<'a>(
        &'a self,
        texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    ) -> Vec<&wgpu::BindGroupLayout> {
        let mut bind_group_layouts = if self.use_textures {
            vec![texture_bind_group_layout]
        } else {
            Vec::new()
        };
        bind_group_layouts.extend(self.uniform_buffers.iter().map(|(_, _, bgl, _)| bgl));
        bind_group_layouts
    }

    pub fn vertex_buffer_layout(&self) -> &wgpu::VertexBufferLayout {
        &self.vertex_buffer_layout
    }

    pub fn update_pipeline(&mut self, render_pipeline: wgpu::RenderPipeline) {
        self.render_pipeline = Some(render_pipeline);
    }

    pub fn name(&self) -> &RenderSceneName {
        &self.name
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &impl VertexBufferWriter,
        indices: &impl IndexBufferWriter,
    ) {
        if let Some((index_buffer, num_indices)) = indices.write_buffer(
            device,
            queue,
            &self.index_buffer,
            self.num_indices,
            wgpu::BufferUsages::INDEX,
            self.num_indices > indices.buffer_len(),
        ) {
            self.index_buffer = index_buffer;
            self.num_indices = num_indices;
        };
        if let Some((vertex_buffer, num_vertices)) = vertices.write_buffer(
            device,
            queue,
            &self.vertex_buffer,
            self.num_vertices,
            wgpu::BufferUsages::VERTEX,
            false,
        ) {
            self.vertex_buffer = vertex_buffer;
            self.num_vertices = num_vertices;
        };
    }

    pub fn write_render_pass<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        texture_bind_group: &'a wgpu::BindGroup,
    ) {
        match self.visibility {
            Visibility::Hidden => return,
            Visibility::Visible => (),
        };
        if let Some(render_pipeline) = &self.render_pipeline {
            render_pass.set_pipeline(render_pipeline);
            let bind_groups = self.bind_groups(texture_bind_group);
            for (i, bind_group) in bind_groups.iter().enumerate() {
                render_pass.set_bind_group(i as u32, bind_group, &[]);
            }
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), self.index_format);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        } else {
            log::warn!("Render pipeline not set for render scene {:?}", self.name);
        }
    }

    pub fn create_uniform_buffer(
        &mut self,
        device: &wgpu::Device,
        label: UniformBufferName,
        contents: &[u8],
        visibility: wgpu::ShaderStages,
    ) {
        let label: UniformBufferName = label.into();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label.as_str()),
            contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label.as_str()),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label.as_str()),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });
        self.uniform_buffers
            .push((label.clone(), buffer, bind_group_layout, bind_group));
    }

    pub fn update_uniform_buffer(
        &self,
        queue: &wgpu::Queue,
        name: &UniformBufferName,
        data: &[u8],
    ) {
        let (_, buffer, _, _) = self
            .uniform_buffers
            .iter()
            .find(|(n, _, _, _)| n == name)
            .expect("Uniform buffer not found");
        queue.write_buffer(buffer, 0, data);
    }
}
