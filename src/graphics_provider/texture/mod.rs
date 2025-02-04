use std::{fs, num::NonZeroU32, path::Path};

use image::GenericImageView;

pub mod exports {
    pub use super::DEFAULT_TEXTURE;
}

pub const DEFAULT_TEXTURE: &str = "Default Texture Provider Texture";

pub struct TextureProvider {
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub bind_group: Option<wgpu::BindGroup>,
    textures: Vec<Texture>,
    current_id: u32,
}
impl TextureProvider {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let bytes = [255; 4];
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture = Texture::from_bytes(device, queue, &bytes, size, Some(DEFAULT_TEXTURE));
        let mut provider = Self {
            bind_group_layout: None,
            bind_group: None,
            textures: Vec::new(),
            current_id: 0,
        };
        provider.register_texture(device, texture);
        provider
    }

    pub fn get_texture_index(&self, label: Option<&str>) -> Option<u32> {
        self.textures
            .iter()
            .enumerate()
            .find(|(_, texture)| texture.label.as_ref().map(|l| l.as_str()) == label)
            .map(|(index, _)| index as u32)
    }

    fn register_texture(
        &mut self,
        device: &wgpu::Device,
        texture: Texture,
    ) -> u32 {
        if let Some(index) = self.get_texture_index(texture.label.as_deref()) {
            return index as u32;
        }
        self.textures.push(texture);
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: NonZeroU32::new(self.current_id + 1),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: NonZeroU32::new(self.current_id + 1),
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(
                        self.textures
                            .iter()
                            .map(|texture| &texture.view)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(
                        self.textures
                            .iter()
                            .map(|texture| &texture.sampler)
                            .collect::<Vec<_>>()
                            .as_slice(),
                    ),
                },
            ],
            label: Some(self.current_id.to_string().as_str()),
        });
        self.bind_group_layout = Some(bind_group_layout);
        self.bind_group = Some(bind_group);
        self.current_id += 1;
        self.current_id - 1
    }

    pub fn create_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,
        label: Option<&str>,
    ) -> u32 {
        if let Some(index) = self.get_texture_index(label) {
            return index as u32;
        }
        let texture = Texture::new(device, queue, path, label);

        self.register_texture(device, texture)
    }
}

pub struct Texture {
    _texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    label: Option<String>,
}

impl Texture {
    fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        size: wgpu::Extent3d,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            _texture: texture,
            view,
            sampler,
            label: label.map(|l| l.to_string()),
        }
    }

    fn new(device: &wgpu::Device, queue: &wgpu::Queue, path: &Path, label: Option<&str>) -> Self {
        let bytes = fs::read(path).expect(&format!("Could not read: '{:?}' for texture {:?}", path, label));
        let img =
            image::load_from_memory(&bytes).expect(&format!("Could not load image: '{:?}", path));

        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        Self::from_bytes(device, queue, &rgba, size, label)
    }
}
