use std::fmt::Debug;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::app::{IndexBuffer, VertexBuffer};
use crate::graphics_provider::{RenderSceneDescriptor, UniformBufferName, Visibility};
use crate::{
    app::{ApplicationEvent, WindowDescriptor},
    graphics::{RenderSceneName, ShaderDescriptor},
};
use winit::window::WindowId;

use super::{Entity, EntityName, EntityType, Scene, SceneName};

use super::ressource_descriptor::{SpriteSheetName, WindowName};

#[derive(Debug)]
pub enum GameEvent<E: ExternalEvent> {
    Timer(Duration),
    Resumed,
    NewWindow(WindowId, WindowName),
    RequestNewWindow(WindowDescriptor, WindowName),
    RenderUpdate(RenderSceneName, VertexBuffer, IndexBuffer),
    NewSpriteSheet(SpriteSheetName, Option<u32>),
    RequestNewSpriteSheet(SpriteSheetName, PathBuf),
    NewRenderScene(RenderSceneName),
    RequestNewRenderScene(
        WindowId,
        RenderSceneName,
        ShaderDescriptor,
        RenderSceneDescriptor,
        ///Initial uniforms for the render scene
        Vec<(UniformBufferName, Vec<u8>, wgpu::ShaderStages)>,
    ),
    RequestSetVisibilityRenderScene(RenderSceneName, Visibility),
    External(E),
    EndGame,
}
impl<E: ExternalEvent> ApplicationEvent for GameEvent<E> {
    fn app_resumed() -> Self {
        Self::Resumed
    }

    fn is_request_new_window<'a>(&'a self) -> Option<(&'a WindowDescriptor, &'a str)> {
        if let Self::RequestNewWindow(window_descriptor, name) = self {
            Some((&window_descriptor, name.as_str()))
        } else {
            None
        }
    }

    fn is_render_update(&self) -> bool {
        match self {
            Self::RenderUpdate(_, _, _) => true,
            _ => false,
        }
    }

    fn consume_render_update(self) -> (RenderSceneName, VertexBuffer, IndexBuffer) {
        if let Self::RenderUpdate(render_scene, vertices, indices) = self {
            (render_scene, vertices, indices)
        } else {
            panic!("You Idiot! Test if it is a render update, before trying to consume the event as one")
        }
    }

    fn is_request_new_texture<'a>(&'a self) -> Option<(&'a Path, &'a str)> {
        if let Self::RequestNewSpriteSheet(label, path) = self {
            Some((path, label.as_str()))
        } else {
            None
        }
    }

    fn is_request_set_visibility_render_scene<'a>(
        &'a self,
    ) -> Option<(&'a RenderSceneName, &'a Visibility)> {
        if let Self::RequestSetVisibilityRenderScene(render_scene, visibility) = self {
            Some((render_scene, visibility))
        } else {
            None
        }
    }

    fn is_request_new_render_scene<'a>(
        &'a self,
    ) -> Option<(
        &'a WindowId,
        &'a RenderSceneName,
        &'a ShaderDescriptor,
        &'a RenderSceneDescriptor,
        &'a [(UniformBufferName, Vec<u8>, wgpu::ShaderStages)],
    )> {
        if let Self::RequestNewRenderScene(
            window_id,
            render_scene,
            shader_descriptor,
            render_scene_descriptor,
            initial_uniforms,
        ) = self
        {
            Some((
                window_id,
                render_scene,
                shader_descriptor,
                render_scene_descriptor,
                initial_uniforms,
            ))
        } else {
            None
        }
    }

    fn new_render_scene(render_scene: &RenderSceneName) -> Self {
        GameEvent::NewRenderScene(render_scene.clone())
    }

    fn new_texture(label: &str, id: Option<u32>) -> Self {
        Self::NewSpriteSheet(label.into(), id)
    }

    fn new_window(id: &WindowId, name: &str) -> Self {
        Self::NewWindow(id.clone(), name.into())
    }

    fn is_quit(&self) -> bool {
        matches!(self, Self::EndGame)
    }
}

pub trait ExternalEvent: Debug + Send {
    type EntityType: EntityType;
    type EntityEvent: Debug;
    fn is_request_render_scene<'a>(&'a self) -> Option<&'a SceneName>;
    fn is_entity_event<'a>(&'a self) -> bool;
    /// Should only be called if is_entity_event returns true
    fn consume_entity_event(self) -> Option<(EntityName, Self::EntityEvent)>;
    fn is_request_set_visibility_scene<'a>(&'a self) -> Option<(&'a SceneName, &'a Visibility)>;
    ///Suspended scenes will now longer update their buffers, but will still be rendered in their
    ///current state
    fn is_request_suspend_scene<'a>(&'a self) -> Option<&'a SceneName>;
    fn is_request_activate_suspended_scene<'a>(&'a self) -> Option<&'a SceneName>;
    ///Deleting a scene will remove it entirely from the game, such that it cannot be rendere again
    fn is_request_delete_scene<'a>(&'a self) -> Option<&'a SceneName>;
    fn is_request_new_scenes<'a>(&'a self) -> bool;
    /// Should only be called if is_request_new_scene returns true
    fn consume_scenes_request(self) -> Option<Vec<Scene<Self>>>
    where
        Self: Sized;
    fn new_scene(scene: &Scene<Self>) -> Self
    where
        Self: Sized;
    fn is_update_uniform_buffer<'a>(&'a self) -> Option<(&'a UniformBufferName, &'a [u8])>;
    fn is_delete_entity<'a>(&'a self) -> Option<(&'a EntityName, &'a SceneName)>;
    fn is_add_entities<'a>(&'a self) -> bool;
    /// Should only be called if is_add_entities returns true
    fn consume_add_entities_request(
        self,
    ) -> Option<(Vec<Box<dyn Entity<Self::EntityType, Self>>>, SceneName)>
    where
        Self: Sized;
    fn is_end_game(&self) -> bool;
}


pub mod example {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    pub enum EmptyEntityType {
        Entity,
    }
    impl EntityType for EmptyEntityType {}
    #[derive(Debug)]
    pub enum EmptyEntityEvent {}
    #[derive(Debug)]
    pub enum EmptyExternalEvent {
        Empty,
    }
    impl ExternalEvent for EmptyExternalEvent {
        type EntityType = EmptyEntityType;
        type EntityEvent = EmptyEntityEvent;
        fn is_end_game(&self) -> bool {
            false
        }
        fn is_request_new_scenes<'a>(&'a self) -> bool {
            false
        }
        fn is_request_render_scene<'a>(&'a self) -> Option<&'a crate::game_engine::SceneName> {
            None
        }
        fn is_entity_event<'a>(&'a self) -> bool {
            false
        }
        fn consume_entity_event(
            self,
        ) -> Option<(crate::game_engine::EntityName, Self::EntityEvent)> {
            None
        }
        fn is_request_delete_scene<'a>(&'a self) -> Option<&'a crate::game_engine::SceneName> {
            None
        }
        fn is_request_suspend_scene<'a>(&'a self) -> Option<&'a crate::game_engine::SceneName> {
            None
        }
        fn is_add_entities<'a>(&'a self) -> bool {
            false
        }
        fn is_request_set_visibility_scene<'a>(
            &'a self,
        ) -> Option<(
            &'a crate::game_engine::SceneName,
            &'a crate::graphics::Visibility,
        )> {
            None
        }
        fn is_request_activate_suspended_scene<'a>(
            &'a self,
        ) -> Option<&'a crate::game_engine::SceneName> {
            None
        }
        fn consume_scenes_request(self) -> Option<Vec<Scene<Self>>>
        where
            Self: Sized,
        {
            None
        }
        fn new_scene(_scene: &Scene<Self>) -> Self
        where
            Self: Sized,
        {
            Self::Empty
        }
        fn consume_add_entities_request(
            self,
        ) -> Option<(
            Vec<Box<dyn crate::game_engine::Entity<Self::EntityType, Self>>>,
            crate::game_engine::SceneName,
        )>
        where
            Self: Sized,
        {
            None
        }
        fn is_delete_entity<'a>(
            &'a self,
        ) -> Option<(
            &'a crate::game_engine::EntityName,
            &'a crate::game_engine::SceneName,
        )> {
            None
        }
        fn is_update_uniform_buffer<'a>(
            &'a self,
        ) -> Option<(&'a crate::graphics::UniformBufferName, &'a [u8])> {
            None
        }
    }
}
