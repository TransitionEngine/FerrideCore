use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{
    app::{IndexBuffer, MouseEvent, VertexBuffer},
    graphics_provider::ShaderDescriptor,
};

use super::{
    app::{EventManager, WindowManager},
    graphics::{GraphicsProvider, RenderSceneName, UniformBufferName},
};
use log::{info, warn};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceId, WindowEvent},
    window::WindowId,
};

use self::camera::Camera;
pub use self::{
    bounding_box::BoundingBox,
    camera::static_camera,
    camera::CameraDescriptor,
    entity::{Entity, EntityName, EntityType},
    game_event::{ExternalEvent, GameEvent},
    ressource_descriptor::{
        RessourceDescriptor, RessourceDescriptorBuilder, SpriteSheetName, WindowName,
    },
    scene::{Scene, SceneName},
    sprite_sheet::{SpritePosition, SpriteSheet, SpriteSheetDimensions, TextureCoordinates},
    velocity_controller::{Direction, VelocityController},
};

mod color;
pub mod example {
    pub use super::color::Color;
    pub use super::game_event::example::*;
    pub use game_state::SimpleGameState;
    pub use vertex::SimpleVertex;

    mod vertex {
        use super::Color;
        use crate::graphics::Vertex;
        use repr_trait::C;
        use threed::Vector;

        #[repr(C)]
        #[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, repr_trait::C)]
        pub struct SimpleVertex {
            position: [f32; 2],
            color: u32,
        }
        impl SimpleVertex {
            pub fn new(position: Vector<f32>, color: Color) -> Self {
                Self {
                    position: [position.x, position.y],
                    color: bytemuck::cast_slice(&color.to_slice())[0],
                }
            }
        }
        const UI_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Uint32];
        impl Vertex for SimpleVertex {
            fn attributes() -> &'static [wgpu::VertexAttribute] {
                &UI_VERTEX_ATTRIBUTES
            }
        }
    }

    mod game_state {
        use crate::game_engine::Scene;

        use super::super::State;
        use super::EmptyExternalEvent;

        pub struct SimpleGameState {
            scene: Option<Scene<EmptyExternalEvent>>,
        }
        impl SimpleGameState {
            pub fn new(scene: Scene<EmptyExternalEvent>) -> Self {
                Self { scene: Some(scene) }
            }
        }
        impl State<EmptyExternalEvent> for SimpleGameState {
            fn handle_event(&mut self, _event: EmptyExternalEvent) -> Vec<EmptyExternalEvent> {
                Vec::new()
            }
            fn start_scenes(mut self) -> (Vec<Scene<EmptyExternalEvent>>, Self) {
                let scenes = if let Some(scene) = self.scene {
                    vec![scene]
                } else {
                    vec![]
                };
                self.scene = None;
                (scenes, self)
            }
        }
    }
}

mod bounding_box;
mod camera;
mod entity;
mod game_event;
mod ressource_descriptor;
mod scene;
mod sprite_sheet;
mod velocity_controller;

pub trait State<E: ExternalEvent> {
    fn handle_event(&mut self, event: E) -> Vec<E>;
    fn start_scenes(self) -> (Vec<Scene<E>>, Self);
}

pub struct Game<E: ExternalEvent, S: State<E>> {
    ressources: RessourceDescriptor,
    active_scenes: Vec<Scene<E>>,
    pending_scenes: Vec<Scene<E>>,
    suspended_scenes: Vec<Scene<E>>,
    window_ids: Vec<(WindowName, WindowId)>,
    window_sizes: Vec<(WindowId, PhysicalSize<u32>)>,
    sprite_sheets: Vec<(SpriteSheetName, SpriteSheet)>,
    cameras: Vec<(SceneName, Camera, UniformBufferName)>,
    cursors: Vec<(DeviceId, WindowId, PhysicalPosition<i32>)>,
    target_fps: u8,
    state: S,
}
impl<E: ExternalEvent, S: State<E>> Game<E, S> {
    pub fn new(ressources: RessourceDescriptor, target_fps: u8, state: S) -> Self {
        let (initial_scenes, state) = state.start_scenes();
        Self {
            ressources,
            pending_scenes: initial_scenes,
            active_scenes: Vec::new(),
            suspended_scenes: Vec::new(),
            window_ids: Vec::new(),
            window_sizes: Vec::new(),
            sprite_sheets: Vec::new(),
            cameras: Vec::new(),
            cursors: Vec::new(),
            target_fps,
            state,
        }
    }

    fn activate_scenes(&mut self, window_manager: &mut WindowManager<GameEvent<E>>) {
        let mut needed_windows = Vec::new();
        let mut scenes_to_discard = Vec::new();
        let mut scenes_to_request = Vec::new();
        for scene in self.pending_scenes.iter() {
            if self
                .active_scenes
                .iter()
                .find(|s| s.name == scene.name)
                .is_some()
            {
                warn!("Scene {:?} already exists. Discarding it", scene.name);
                scenes_to_discard.push(scene.name.clone());
                continue;
            }
            if let Some((_, id)) = self
                .window_ids
                .iter()
                .find(|(existing_window, _)| scene.target_window == *existing_window)
            {
                scenes_to_request.push((
                    id.clone(),
                    scene.render_scene.clone(),
                    scene.name.clone(),
                    scene.shader_descriptor.clone(),
                ));
            } else {
                if !needed_windows.contains(&scene.target_window) {
                    needed_windows.push(scene.target_window.clone());
                }
            }
        }
        for (window_id, render_scene, scene, shader_descriptor) in scenes_to_request {
            self.request_render_scene(
                &window_id,
                window_manager,
                render_scene,
                scene,
                shader_descriptor,
            );
        }
        for window_name in needed_windows.iter() {
            let window_descriptor = &self
                .ressources
                .get_window(&window_name)
                .expect(&format!("No ressources provided for {:?}", window_name));
            window_manager.send_event(GameEvent::RequestNewWindow(
                window_descriptor.clone(),
                window_name.clone(),
            ));
        }
        self.pending_scenes
            .retain_mut(|s| !scenes_to_discard.contains(&s.name));
    }

    fn request_render_scene(
        &mut self,
        target_window: &WindowId,
        window_manager: &mut WindowManager<GameEvent<E>>,
        render_scene: RenderSceneName,
        scene: SceneName,
        shader_descriptor: ShaderDescriptor,
    ) {
        let (camera, render_scene_descriptor) = self.ressources.get_render_scene(&render_scene);
        let mut uniform_buffers: Vec<(UniformBufferName, Vec<u8>, wgpu::ShaderStages)> =
            shader_descriptor
                .uniforms
                .iter()
                .map(|name| {
                    self.ressources
                        .get_uniform(&(*name).into())
                        .expect(&format!(
                            "Did not specify UniformBuffer {:?} in RessourceDescriptor",
                            name
                        ))
                })
                .collect();
        if let Some(camera_descriptor) = camera {
            let camera: Camera = (&camera_descriptor).into();
            let uniform_name = &format!("{:?} camera", render_scene.as_str());
            let bytes = camera.as_bytes();
            self.cameras
                .push((scene.clone(), camera, uniform_name.into()));
            uniform_buffers.push((uniform_name.into(), bytes, wgpu::ShaderStages::VERTEX));
        };
        window_manager.send_event(GameEvent::RequestNewRenderScene(
            target_window.clone(),
            render_scene,
            shader_descriptor,
            render_scene_descriptor,
            uniform_buffers,
        ));
    }

    fn request_sprite_sheet(
        &self,
        name: &SpriteSheetName,
        window_manager: &mut WindowManager<GameEvent<E>>,
    ) {
        let path = &self.ressources.get_sprite_sheet(&name).0;
        window_manager.send_event(GameEvent::RequestNewSpriteSheet(name.clone(), path.clone()));
    }

    fn get_window_name(&self, id: &WindowId) -> Option<&WindowName> {
        self.window_ids
            .iter()
            .find(|(_, i)| i == id)
            .map(|(name, _)| name)
    }
}
impl<E: ExternalEvent + 'static, S: State<E>> EventManager<GameEvent<E>> for Game<E, S> {
    fn window_event(
        &mut self,
        window_manager: &mut WindowManager<GameEvent<E>>,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        id: &winit::window::WindowId,
        event: &winit::event::WindowEvent,
    ) -> bool
    where
        Self: Sized,
    {
        match event {
            WindowEvent::Resized(size) => {
                let window_size = self.window_sizes.iter_mut().find(|(i, _)| i == id);
                if let Some((_, s)) = window_size {
                    *s = *size
                } else {
                    self.window_sizes.push((id.clone(), *size));
                }
            }
            WindowEvent::CursorEntered { device_id } => {
                self.cursors.push((
                    device_id.clone(),
                    id.clone(),
                    PhysicalPosition::new(0, 0),
                ));
            }
            WindowEvent::CursorLeft { device_id } => {
                self.cursors.retain(|(id, _, _)| id != device_id);
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Some((_, _, cursor_position)) = self
                    .cursors
                    .iter_mut()
                    .find(|(device, window, _)| device_id == device && window == id)
                {
                    if let Some((_, size)) = self.window_sizes.iter().find(|(i, _)| i == id) {
                        let position = PhysicalPosition::new(
                            (position.x - size.width as f64 / 2.0) as i32,
                            (position.y - size.height as f64 / 2.0) as i32,
                        );
                        *cursor_position = position;
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button,
                device_id,
            } => match self.get_window_name(id) {
                Some(window_name) => {
                    let window_name = window_name.clone();
                    for scene in self
                        .active_scenes
                        .iter_mut()
                        .filter(|scene| scene.target_window == window_name)
                    {
                        if let Some((_, _, position)) = self
                            .cursors
                            .iter()
                            .find(|(device, window, _)| device == device_id && window == id)
                        {
                            let events = scene.handle_mouse_input(&MouseEvent {
                                state: *state,
                                button: *button,
                                position: *position,
                            });
                            for event in events {
                                window_manager.send_event(GameEvent::External(event));
                            }
                        }
                    }
                }
                None => {
                    warn!("No window name found for window id {:?}", id)
                }
            },
            WindowEvent::KeyboardInput { event, .. } => {
                match self.get_window_name(id) {
                    Some(window_name) => {
                        let window_name = window_name.clone();
                        for scene in self
                            .active_scenes
                            .iter_mut()
                            .filter(|scene| scene.target_window == window_name)
                        {
                            let events = scene.handle_key_input(event);
                            if let Some((_, camera, _)) =
                                self.cameras.iter_mut().find(|(n, _, _)| n == &scene.name)
                            {
                                camera.handle_key_input(event);
                            }
                            for event in events {
                                window_manager.send_event(GameEvent::External(event));
                            }
                        }
                    }
                    None => {
                        warn!("No window name found for window id {:?}", id)
                    }
                };
            }
            _ => {}
        }
        true
    }

    fn user_event(
        &mut self,
        window_manager: &mut WindowManager<GameEvent<E>>,
        graphics_provider: &mut GraphicsProvider,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        event: GameEvent<E>,
    ) where
        Self: Sized,
    {
        match event {
            GameEvent::Resumed => {
                self.activate_scenes(window_manager);

                let ns_per_frame = 1e9 / (self.target_fps as f64);
                let frame_duration = Duration::from_nanos(ns_per_frame as u64);
                let timer_event_loop = window_manager.create_event_loop_proxy();
                thread::spawn(move || {
                    let mut last_update = Instant::now();
                    loop {
                        match timer_event_loop.send_event(GameEvent::Timer(last_update.elapsed())) {
                            Ok(()) => {}
                            Err(_) => break,
                        };
                        last_update = Instant::now();
                        thread::sleep(frame_duration);
                    }
                });
            }
            GameEvent::NewWindow(id, name) => {
                self.window_ids.push((name.clone(), id.clone()));
                for i in 0..self.pending_scenes.len() {
                    let scene = &self.pending_scenes[i];
                    if scene.target_window == name {
                        self.request_render_scene(
                            &id,
                            window_manager,
                            scene.render_scene.clone(),
                            scene.name.clone(),
                            scene.shader_descriptor.clone(),
                        );
                    }
                }
            }
            GameEvent::NewRenderScene(render_scene) => {
                let index = self
                    .pending_scenes
                    .iter()
                    .position(|scene| scene.render_scene == render_scene)
                    .expect("Scene Vanished before getting created fully");
                for sprite_sheet in self.pending_scenes[index]
                    .entities
                    .iter()
                    .map(|e| e.sprite_sheets())
                    .flatten()
                {
                    self.request_sprite_sheet(&sprite_sheet, window_manager);
                }
                let scene = self.pending_scenes.remove(index);
                window_manager.send_event(GameEvent::External(E::new_scene(&scene)));
                self.active_scenes.push(scene);
                self.active_scenes.sort_by_key(|s| s.z_index);
            }
            GameEvent::NewSpriteSheet(label, None) => {
                panic!("Could not load SpriteSheet '{:?}'", label)
                // self.request_sprite_sheet(label, window_manager)
            }
            GameEvent::NewSpriteSheet(label, Some(id)) => {
                if self
                    .sprite_sheets
                    .iter()
                    .find(|(l, _)| label == *l)
                    .is_none()
                {
                    let dimensions = &self.ressources.get_sprite_sheet(&label).1;
                    let sprite_sheet = SpriteSheet::new(id, dimensions);
                    self.sprite_sheets.push((label.clone(), sprite_sheet));
                }
            }
            GameEvent::Timer(delta_t) => {
                for scene in self
                    .active_scenes
                    .iter_mut()
                    .chain(self.suspended_scenes.iter_mut())
                {
                    let mut vertices = VertexBuffer::new();
                    let mut indices = IndexBuffer::new();
                    let entities = &mut scene.entities;
                    entities.sort_by(|a, b| a.z().partial_cmp(&b.z()).expect("NaN NaN NaN"));
                    for i in 0..entities.len() {
                        let (left, right) = entities.split_at_mut(i);
                        let (entity, right) = right.split_first_mut().expect("i out of bounds");
                        let interactions = left.iter().chain(right.iter()).map(|e| &*e).collect();
                        let events = entity.update(&interactions, &delta_t, &scene.name);
                        for event in events {
                            window_manager.send_event(GameEvent::External(event))
                        }
                        let sprite_sheets = entity
                            .sprite_sheets()
                            .iter()
                            .map(|entity_sprite_sheet| {
                                self.sprite_sheets
                                    .iter()
                                    .find(|(l, _)| l == *entity_sprite_sheet)
                                    .map(|(_, s)| s)
                            })
                            .collect();
                        entity.render(&mut vertices, &mut indices, sprite_sheets);
                    }
                    if let Some((_, camera, camera_name)) =
                        self.cameras.iter_mut().find(|(n, _, _)| n == &scene.name)
                    {
                        match camera.update(entities.iter().map(|e| &*e).collect(), &delta_t) {
                            Ok(()) => {}
                            Err(err) => info!("Camera update failed: {}", err),
                        };
                        graphics_provider.update_uniform_buffer(camera_name, &camera.as_bytes());
                    }
                    window_manager.send_event(GameEvent::RenderUpdate(
                        scene.render_scene.clone(),
                        vertices,
                        indices,
                    ));
                }
            }
            GameEvent::External(event) => {
                println!("EXTERN EVENT: {:?}", event);
                if event.is_request_new_scenes() {
                    info!("Creating new Scenes");
                    let scenes = event
                        .consume_scenes_request()
                        .expect("Bad implementation of ExternalEvent::is_request_new_scenes() should only return true, if ExternalEvent::consume_scenes_request() returns Some(scenes)");
                    self.pending_scenes.extend(scenes);
                    self.activate_scenes(window_manager);
                    return;
                }
                if event.is_add_entities() {
                    info!("Adding new entities to scene");
                    let (mut entities, scene) = event
                        .consume_add_entities_request()
                        .expect("Bad implementation of ExternalEvent::is_add_entities() should only return true, if ExternalEvent::consume_add_entities_request() returns Some(entities, scene)");
                    let scene = &mut self
                        .active_scenes
                        .iter_mut()
                        .find(|s| s.name == scene)
                        .unwrap_or_else(|| {
                            self.suspended_scenes
                                .iter_mut()
                                .find(|s| s.name == scene)
                                .expect(&format!("Found no active nor suspended scene {:?}", scene))
                        });
                    scene.entities.append(&mut entities);
                    return;
                }
                if let Some((scene, visibility)) = event.is_request_set_visibility_scene() {
                    let render_scene = &self
                        .active_scenes
                        .iter()
                        .find(|s| s.name == *scene)
                        .unwrap_or_else(|| {
                            self.suspended_scenes
                                .iter_mut()
                                .find(|s| s.name == *scene)
                                .expect(&format!("Found no active nor suspended scene {:?}", scene))
                        })
                        .render_scene;
                    window_manager.send_event(GameEvent::RequestSetVisibilityRenderScene(
                        render_scene.clone(),
                        visibility.clone(),
                    ));
                }
                if let Some(suspendable_scene) = event.is_request_suspend_scene() {
                    info!("Suspending Scene {:?}", suspendable_scene);
                    if let Some(index) = self
                        .active_scenes
                        .iter()
                        .position(|s| s.name == *suspendable_scene)
                    {
                        let scene = self.active_scenes.remove(index);
                        self.suspended_scenes.push(scene);
                        self.cameras
                            .iter_mut()
                            .filter(|(s, _, _)| s == suspendable_scene)
                            .for_each(|(_, camera, _)| camera.reset_offset());
                    } else {
                        warn!(
                            "Tried to suspend Scene {:?}, but it is not active",
                            suspendable_scene
                        );
                    }
                }
                if let Some(activatable_scene) = event.is_request_activate_suspended_scene() {
                    info!("Activating Scene: {:?}", activatable_scene);
                    if let Some(index) = self
                        .suspended_scenes
                        .iter()
                        .position(|s| s.name == *activatable_scene)
                    {
                        let scene = self.suspended_scenes.remove(index);
                        self.active_scenes.push(scene);
                        self.active_scenes.sort_by_key(|s| s.z_index);
                    } else {
                        warn!(
                            "Tried to activate suspended Scene {:?}, but it is not suspended",
                            activatable_scene
                        );
                    }
                }
                if let Some(deletable_scene) = event.is_request_delete_scene() {
                    info!("Deleting Scene {:?}", deletable_scene);
                    if let Some(active_index) = self
                        .active_scenes
                        .iter()
                        .position(|s| s.name == *deletable_scene)
                    {
                        let scene = self.active_scenes.remove(active_index);
                        graphics_provider.remove_render_scene(&scene.render_scene);
                    } else if let Some(suspended_index) = self
                        .suspended_scenes
                        .iter()
                        .position(|s| s.name == *deletable_scene)
                    {
                        let scene = self.suspended_scenes.remove(suspended_index);
                        graphics_provider.remove_render_scene(&scene.render_scene);
                    } else {
                        warn!(
                            "Tried to delete Scene {:?}, but its neither active nor suspended",
                            deletable_scene
                        );
                    }
                    self.cameras
                        .retain(|(scene_name, _, _)| scene_name != deletable_scene);
                }
                if let Some((uniform_name, contents)) = event.is_update_uniform_buffer() {
                    graphics_provider.update_uniform_buffer(uniform_name, contents);
                }
                if let Some((entity, scene)) = event.is_delete_entity() {
                    info!("Deleting Entiy {:?} from Scene {:?}", entity, scene);
                    let scene = self
                        .active_scenes
                        .iter_mut()
                        .find(|s| s.name == *scene)
                        .unwrap_or_else(|| {
                            self.suspended_scenes
                                .iter_mut()
                                .find(|s| s.name == *scene)
                                .expect(&format!("Found no active nor suspended scene {:?}", scene))
                        });
                    scene.entities.retain(|e| e.name() != entity);
                    for e in scene.entities.iter_mut() {
                        e.delete_child_entity(entity);
                    }
                }
                if let Some(scene) = event.is_request_render_scene() {
                    if let Some(scene) = self.active_scenes.iter_mut().find(|s| s.name == *scene) {
                        scene.simple_render(&self.sprite_sheets, window_manager)
                    } else {
                        warn!("Tried to render Scene {:?}, but it is not active", scene);
                    }
                }
                if event.is_end_game() {
                    window_manager.send_event(GameEvent::EndGame);
                    return;
                }
                let response_events = if event.is_entity_event() {
                    let (target, event) = event.consume_entity_event().expect("unreachable");
                    let mut target_entity = None;
                    for scene in &mut self.active_scenes {
                        match scene.entities.iter_mut().find(|e| e.name() == &target) {
                            Some(entity) => {
                                target_entity = Some(entity);
                                break;
                            }
                            None => continue,
                        }
                    }
                    if let Some(target) = target_entity {
                        target.handle_event(event)
                    } else {
                        warn!(
                            "Tried to send event to entity {:?}, but it does not exist in an active scene",
                            target
                        );
                        vec![]
                    }
                } else {
                    self.state.handle_event(event)
                };

                for event in response_events {
                    window_manager.send_event(GameEvent::External(event));
                }
            }
            _ => {}
        }
    }
}
