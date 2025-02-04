use std::{fmt::Debug, path::Path};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Fullscreen, WindowId},
};

pub mod winit_reexports {
    pub use super::window_descriptor::winit_reexports::*;
    pub use super::event_manager::winit_reexports::*;
    pub use winit::keyboard;
}


mod window_descriptor;
use window_descriptor::WindowDescriptor;
mod event_manager;
use event_manager::EventManager;
mod window_manager;
use window_manager::WindowManager;
mod buffer;
use buffer::{IndexBuffer, VertexBuffer};

use crate::graphics::{
    GraphicsProvider, RenderSceneDescriptor, RenderSceneName, ShaderDescriptor, UniformBufferName,
    Visibility,
};

pub mod exports {
    pub use super::window_descriptor::exports::*;
    pub use super::window_manager::exports::*;
    pub use super::buffer::exports::*;
    pub use super::event_manager::exports::*;
    pub use super::{ApplicationEvent, ManagerApplication};
}

pub struct ManagerApplication<E: ApplicationEvent + 'static, M: EventManager<E>> {
    event_manager: M,
    window_manager: WindowManager<E>,
    graphics_provider: GraphicsProvider,
}

impl<'a, E: ApplicationEvent + 'static, M: EventManager<E>> ApplicationHandler<E>
    for ManagerApplication<E, M>
{
    fn resumed(&mut self, _active_loop: &ActiveEventLoop) {
        self.window_manager.send_event(E::app_resumed());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self
            .event_manager
            .window_event(&mut self.window_manager, event_loop, &id, &event)
        {
            match event {
                WindowEvent::CloseRequested => {
                    if self.window_manager.amount_windows() == 1 {
                        event_loop.exit();
                    } else {
                        self.graphics_provider.remove_window(&id);
                        self.window_manager.remove_window(&id);
                    }
                }
                WindowEvent::Resized(size) => self.graphics_provider.resize_window(&id, &size),
                WindowEvent::ScaleFactorChanged { .. } => {
                    //TODO: I think the window will be resized  on its own, which fires a Resized event
                }
                WindowEvent::RedrawRequested => {
                    self.graphics_provider.render_window(&id);
                    self.window_manager
                        .get_window(&id)
                        .expect("The window dissapeared")
                        .request_redraw();
                }
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::F11),
                            ..
                        },
                    ..
                } => {
                    let window = self
                        .window_manager
                        .get_window(&id)
                        .expect("The window dissapeared");
                    match window.fullscreen() {
                        Some(Fullscreen::Borderless(_)) => {
                            window.set_fullscreen(None);
                        }
                        _ => {
                            window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: E) {
        match event.is_request_new_window() {
            Some((window_descriptor, name)) => {
                self.create_window(window_descriptor, event_loop, name)
            }
            None => {}
        };
        if event.is_render_update() {
            let (render_scene, vertices, indices) = event.consume_render_update();
            self.graphics_provider
                .update_scene(&render_scene, &vertices, &indices);
            return;
        }
        match event.is_request_new_texture() {
            Some((path, label)) => {
                let id = self.graphics_provider.create_texture(path, label);
                self.window_manager.send_event(E::new_texture(label, id));
            }
            None => {}
        }
        match event.is_request_new_render_scene() {
            Some((
                window_id,
                render_scene,
                shader_descriptor,
                render_scene_descriptor,
                initial_uniforms,
            )) => {
                self.graphics_provider.add_render_scene(
                    window_id,
                    render_scene.clone(),
                    shader_descriptor.clone(),
                    render_scene_descriptor.clone(),
                    initial_uniforms,
                );
                self.window_manager
                    .send_event(E::new_render_scene(render_scene))
            }
            None => {}
        }
        match event.is_request_set_visibility_render_scene() {
            Some((render_scene, visibility)) => {
                self.graphics_provider
                    .set_visibility_render_scene(&render_scene, visibility);
            }
            None => {}
        }
        if event.is_quit() {
            event_loop.exit();
            return;
        }

        self.event_manager.user_event(
            &mut self.window_manager,
            &mut self.graphics_provider,
            event_loop,
            event,
        );
    }
}

impl<'a, E: ApplicationEvent + 'static, M: EventManager<E>> ManagerApplication<E, M> {
    pub fn new(event_manager: M) -> Self {
        Self {
            event_manager,
            window_manager: Default::default(),
            graphics_provider: GraphicsProvider::new(),
        }
    }

    fn create_window(
        &mut self,
        descriptor: &WindowDescriptor,
        active_loop: &ActiveEventLoop,
        name: &str,
    ) {
        let window = active_loop
            .create_window(descriptor.get_attributes(active_loop))
            .expect("OS says: 'No more windows for you'");
        self.window_manager
            .send_event(E::new_window(&window.id(), name));
        self.graphics_provider.init_window(&window);
        // window.request_redraw();
        self.window_manager.add_window(window);
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::<E>::with_user_event()
            .build()
            .expect("No loop for you");
        let event_loop_proxy = event_loop.create_proxy();
        self.window_manager.set_event_loop(event_loop_proxy);

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(self).expect("No App for you");
    }
}

pub trait ApplicationEvent: Debug {
    fn app_resumed() -> Self;
    fn new_window(id: &WindowId, name: &str) -> Self;
    fn new_texture(label: &str, id: Option<u32>) -> Self;
    fn new_render_scene(render_scene: &RenderSceneName) -> Self;
    fn is_request_new_window<'a>(&'a self) -> Option<(&'a WindowDescriptor, &'a str)>;
    fn is_render_update(&self) -> bool;
    fn consume_render_update(self) -> (RenderSceneName, VertexBuffer, IndexBuffer);
    fn is_request_new_texture<'a>(&'a self) -> Option<(&'a Path, &'a str)>;
    fn is_request_new_render_scene<'a>(
        &'a self,
    ) -> Option<(
        &'a WindowId,
        &'a RenderSceneName,
        &'a ShaderDescriptor,
        &'a RenderSceneDescriptor,
        &'a [(UniformBufferName, Vec<u8>, wgpu::ShaderStages)],
    )>;
    fn is_request_set_visibility_render_scene<'a>(
        &'a self,
    ) -> Option<(&'a RenderSceneName, &'a Visibility)>;
    fn is_quit(&self) -> bool;
}
