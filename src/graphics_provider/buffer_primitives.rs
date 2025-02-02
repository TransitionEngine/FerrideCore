use std::fmt::Debug;

pub trait Vertex:
    Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable + repr_trait::C
{
    fn describe_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::attributes(),
        }
    }

    fn attributes() -> &'static [wgpu::VertexAttribute];
}
pub trait Index: Debug + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable {
    fn index_format() -> wgpu::IndexFormat;
}
impl Index for u16 {
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
impl Index for u32 {
    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint32
    }
}

