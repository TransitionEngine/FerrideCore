#![allow(deprecated)]
use std::fs;
use std::path::Path;

use wgpu::rwh::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::window::{Window, WindowId};

mod buffer_primitives;
pub use buffer_primitives::{Index, Vertex};

mod surface;
use surface::{Surface, WindowSurface};

mod shader_descriptor;
pub use shader_descriptor::ShaderDescriptor;

mod texture;
pub use texture::DEFAULT_TEXTURE;
use texture::TextureProvider;

mod buffer_writer;
pub use buffer_writer::{BufferWriter, IndexBufferWriter, VertexBufferWriter};

mod render_scene;
use render_scene::RenderScene;
pub use render_scene::{RenderSceneDescriptor, RenderSceneName, UniformBufferName};

#[derive(Debug, Clone)]
pub enum Visibility {
    Visible,
    Hidden,
}

pub struct GraphicsProvider {
    instance: wgpu::Instance,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    ///One to one relationship
    surfaces: Vec<(WindowId, Box<dyn WindowSurface>)>,
    ///One to many relationship
    render_scenes: Vec<(WindowId, RenderScene, wgpu::ShaderModule, ShaderDescriptor)>,
    texture_provider: Option<TextureProvider>,
    uniform_buffers: Vec<(RenderSceneName, UniformBufferName)>,
}
impl GraphicsProvider {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        Self {
            instance,
            adapter: None,
            device: None,
            queue: None,
            surfaces: Vec::new(),
            render_scenes: Vec::new(),
            uniform_buffers: Vec::new(),
            texture_provider: None,
        }
    }

    pub fn set_visibility_render_scene(&mut self, render_scene: &RenderSceneName, visibility: &Visibility) {
        if let Some((_, scene, _, _)) = self.render_scenes.iter_mut().find(|(_, r, _, _)| r.name() == render_scene) {
            scene.set_visibility(visibility);
        }
    }

    pub fn remove_render_scene(&mut self, render_scene: &RenderSceneName) {
        self.render_scenes
            .retain(|(_, r, _, _)| r.name() != render_scene);
    }

    pub fn get_window(&self, render_scene: &RenderSceneName) -> Option<&WindowId> {
        self.render_scenes
            .iter()
            .find(|(_, scene, _, _)| render_scene == scene.name())
            .map(|(window_id, _, _, _)| window_id)
    }

    fn init(&mut self, surface: &wgpu::Surface) {
        let adapter = futures::executor::block_on(self.instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .expect("Buy a new GPU. Not all prerequisites met");

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING,
                required_limits: wgpu::Limits {
                    // max_vertex_attributes: 32,
                    ..Default::default()
                },
                label: None,
            },
            None, // Trace path
        ))
        .expect("Buy a new GPU. Not all prerequisites met");
        self.texture_provider = Some(TextureProvider::new(&device, &queue));
        self.adapter = Some(adapter);
        self.device = Some(device);
        self.queue = Some(queue);
    }

    pub fn init_window(&mut self, window: &Window) {
        let size = window.inner_size();
        //#Safety
        //
        //Should be safe if surface discarded when window is destroyed
        let surface = unsafe {
            self.instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: window
                        .raw_display_handle()
                        .expect("The window has no display handle"),
                    raw_window_handle: window
                        .raw_window_handle()
                        .expect("The window has no window handle"),
                })
        }
        .expect("Could not create a surface");

        if self.adapter.is_none() {
            self.init(&surface);
        }

        let capabilities = surface.get_capabilities(
            &self
                .adapter
                .as_ref()
                .expect("The surface is not compatible with the adapter"),
        );
        let format = capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .or(Some(capabilities.formats[0]))
            .expect("No compatible format found");
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        self.surfaces.push((
            window.id(),
            Box::new(Surface {
                wgpu_surface: surface,
                config,
            }),
        ));
    }

    pub fn resize_window(&mut self, id: &WindowId, new_size: &winit::dpi::PhysicalSize<u32>) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let Some(device) = &self.device {
                surface.resize(new_size, device);
            }
        }
    }

    pub fn render_window(&mut self, id: &WindowId) {
        if let Some((_, surface)) = self.surfaces.iter_mut().find(|(i, _)| i == id) {
            if let (Some(device), Some(queue), Some(texture_provider)) =
                (&self.device, &self.queue, &self.texture_provider)
            {
                let texture_bind_group =
                    texture_provider.bind_group.as_ref().expect("No bind group");
                let render_scenes = self
                    .render_scenes
                    .iter()
                    .filter_map(|(i, s, _, _)| if i == id { Some(s) } else { None })
                    .collect::<Vec<_>>();
                surface.render(device, queue, &render_scenes, texture_bind_group);
            }
        }
    }

    /// Update the vertex and index buffers of a window
    pub fn update_scene(
        &mut self,
        render_scene: &RenderSceneName,
        vertices: &impl VertexBufferWriter,
        indices: &impl IndexBufferWriter,
    ) {
        if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
            for render_scene in self.render_scenes.iter_mut().filter_map(|(_, s, _, _)| {
                if render_scene == s.name() {
                    Some(s)
                } else {
                    None
                }
            }) {
                render_scene.update(device, queue, vertices, indices)
            }
        }
    }

    pub fn add_render_scene(
        &mut self,
        window_id: &WindowId,
        render_scene_name: RenderSceneName,
        shader_descriptor: ShaderDescriptor,
        render_scene_descriptor: RenderSceneDescriptor,
        initial_uniforms: &[(UniformBufferName, Vec<u8>, wgpu::ShaderStages)],
    ) {
        let device = self.device.as_ref().expect("The device vanished");

        if let (Some((_, surface)), Some(texture_provider)) = (
            self.surfaces.iter().find(|(id, _)| id == window_id),
            &self.texture_provider,
        ) {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("Shader Module {:?}", shader_descriptor.file)),
                source: wgpu::ShaderSource::Wgsl(
                    fs::read_to_string(shader_descriptor.file)
                        .expect(&format!("Could not load '{}'\n", shader_descriptor.file))
                        .into(),
                ),
            });
            let mut render_scene =
                RenderScene::new(render_scene_name.clone(), device, render_scene_descriptor);
            for (uniform, content, visibility) in initial_uniforms {
                render_scene.create_uniform_buffer(
                    device,
                    uniform.clone(),
                    content,
                    visibility.clone(),
                );
                self.uniform_buffers
                    .push((render_scene_name.clone(), uniform.clone()));
            }
            let bind_groups_layouts = render_scene.bind_group_layouts(
                texture_provider
                    .bind_group_layout
                    .as_ref()
                    .expect("Default Texture vanished"),
            );
            let render_pipeline = surface.create_render_pipeline(
                device,
                &bind_groups_layouts,
                &shader,
                &shader_descriptor,
                render_scene.vertex_buffer_layout().clone(),
            );
            render_scene.update_pipeline(render_pipeline);
            self.render_scenes
                .push((window_id.clone(), render_scene, shader, shader_descriptor));
        } else {
            panic!("No surface on window {:?}", window_id)
        }
    }

    pub fn remove_window(&mut self, id: &WindowId) {
        self.surfaces.retain(|(i, _)| i != id);
        let render_scenes_to_delete = self
            .render_scenes
            .iter()
            .filter_map(|(i, s, _, _)| if i == id { Some(s.name()) } else { None })
            .collect::<Vec<_>>();
        self.uniform_buffers
            .retain(|(r, _)| !render_scenes_to_delete.contains(&r));
        self.render_scenes.retain(|(i, _, _, _)| i != id);
    }

    pub fn create_texture(&mut self, path: &Path, label: &str) -> Option<u32> {
        if let (Some(device), Some(queue), Some(texture_provider)) =
            (&self.device, &self.queue, &mut self.texture_provider)
        {
            let index = texture_provider.create_texture(device, queue, path, Some(label));
            let texture_bind_group_layout = texture_provider
                .bind_group_layout
                .as_ref()
                .expect("No texture bind group layout");
            self.render_scenes
                .iter_mut()
                .filter(|(_, s, _, _)| s.use_textures())
                .for_each(|(window_id, render_scene, shader, shader_descriptor)| {
                    if let Some((_, surface)) = self.surfaces.iter().find(|(id, _)| id == window_id)
                    {
                        let bind_groups_layouts =
                            render_scene.bind_group_layouts(texture_bind_group_layout);
                        let render_pipeline = surface.create_render_pipeline(
                            device,
                            &bind_groups_layouts,
                            shader,
                            shader_descriptor,
                            render_scene.vertex_buffer_layout().clone(),
                        );
                        render_scene.update_pipeline(render_pipeline);
                    }
                });
            Some(index)
        } else {
            None
        }
    }

    pub fn create_uniform_buffer(
        &mut self,
        label: impl Into<UniformBufferName>,
        contents: &[u8],
        visibility: wgpu::ShaderStages,
        target_render_scene: &RenderSceneName,
    ) {
        let device = self.device.as_ref().expect("The device vanished");
        if let Some((_, render_scene, _, _)) = self
            .render_scenes
            .iter_mut()
            .find(|(_, s, _, _)| s.name() == target_render_scene)
        {
            let label = label.into();
            render_scene.create_uniform_buffer(device, label.clone(), contents, visibility);
            self.uniform_buffers
                .push((target_render_scene.clone(), label));
        } else {
            panic!(
                "Could not find any {:?} to attach {:?} to",
                target_render_scene,
                label.into()
            );
        }
    }

    pub fn update_uniform_buffer(&self, label: &UniformBufferName, contents: &[u8]) {
        if let Some((target_render_scene, _)) =
            self.uniform_buffers.iter().find(|(_, u)| u == label)
        {
            let (_, render_scene, _, _) = self
                .render_scenes
                .iter()
                .find(|(_, s, _, _)| s.name() == target_render_scene)
                .expect(&format!("RenderScene {:?} vanished", target_render_scene));
            let queue = self.queue.as_ref().expect("The queue vanished");
            render_scene.update_uniform_buffer(queue, label, contents);
        }
    }
}
