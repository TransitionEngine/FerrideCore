use crate::{
    app::{IndexBuffer, MouseEvent, VertexBuffer, WindowManager},
    create_name_struct,
    graphics::{RenderSceneName, ShaderDescriptor},
};
use winit::event::KeyEvent;

use super::{
    entity::Entity, ressource_descriptor::WindowName, ExternalEvent, GameEvent, SpriteSheet,
    SpriteSheetName,
};

pub mod exports {
    pub use super::{Scene, SceneName};
}

create_name_struct!(SceneName);

#[derive(Debug)]
pub struct Scene<E: ExternalEvent> {
    pub name: SceneName,
    pub shader_descriptor: ShaderDescriptor,
    pub render_scene: RenderSceneName,
    pub target_window: WindowName,
    pub entities: Vec<Box<dyn Entity<E::EntityType, E>>>,
    pub z_index: i32,
}
impl<E: ExternalEvent> Scene<E> {
    pub fn simple_render(
        &mut self,
        sprite_sheets: &[(SpriteSheetName, SpriteSheet)],
        window_manager: &mut WindowManager<GameEvent<E>>,
    ) {
        let mut vertices = VertexBuffer::new();
        let mut indices = IndexBuffer::new();
        let entities = &mut self.entities;
        entities.sort_by(|a, b| a.z().partial_cmp(&b.z()).expect("NaN NaN NaN"));
        for i in 0..entities.len() {
            let (_, right) = entities.split_at_mut(i);
            let (entity, _) = right.split_first_mut().expect("i out of bounds");
            let sprite_sheets = entity
                .sprite_sheets()
                .iter()
                .map(|entity_sprite_sheet| {
                    sprite_sheets
                        .iter()
                        .find(|(l, _)| l == *entity_sprite_sheet)
                        .map(|(_, s)| s)
                })
                .collect();
            entity.render(&mut vertices, &mut indices, sprite_sheets);
        }
        window_manager.send_event(GameEvent::RenderUpdate(
            self.render_scene.clone(),
            vertices,
            indices,
        ));
    }

    pub fn handle_key_input(&mut self, input: &KeyEvent) -> Vec<E> {
        let mut events = vec![];
        for entity in self.entities.iter_mut() {
            events.append(&mut entity.handle_key_input(input));
        }
        events
    }

    pub fn handle_mouse_input(&mut self, input: &MouseEvent) -> Vec<E> {
        let mut events = vec![];
        for entity in self.entities.iter_mut() {
            events.append(&mut entity.handle_mouse_input(input));
        }
        events
    }
}
