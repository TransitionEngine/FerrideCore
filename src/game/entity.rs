use crate::{
    app::{IndexBuffer, MouseEvent, VertexBuffer},
    create_name_struct,
};
use std::{fmt::Debug, time::Duration};
use threed::Vector;
use winit::event::KeyEvent;

use super::{
    ressource_descriptor::SpriteSheetName, sprite_sheet::SpriteSheet, BoundingBox, ExternalEvent,
    SceneName,
};

create_name_struct!(EntityName);

pub trait EntityType: PartialEq + Debug {}

pub trait Entity<T: EntityType, E: ExternalEvent>: Debug + Send {
    fn update(
        &mut self,
        _entities: &Vec<&Box<dyn Entity<T, E>>>,
        _delta_t: &Duration,
        _scene: &SceneName,
    ) -> Vec<E> {
        vec![]
    }
    fn render(
        &mut self,
        vertices: &mut VertexBuffer,
        indices: &mut IndexBuffer,
        sprite_sheet: Vec<Option<&SpriteSheet>>,
    );
    fn sprite_sheets(&self) -> Vec<&SpriteSheetName>;
    fn handle_key_input(&mut self, _input: &KeyEvent) -> Vec<E> { 
        vec![]
    }
    fn handle_mouse_input(&mut self, _input: &MouseEvent) -> Vec<E> {
        vec![]
    }
    fn name(&self) -> &EntityName;
    fn bounding_box(&self) -> BoundingBox;
    fn entity_type(&self) -> T;

    fn z(&self) -> f32 {
        self.position().z
    }
    fn position(&self) -> Vector<f32> {
        self.bounding_box().anchor
    }
    fn delete_child_entity(&mut self, _name: &EntityName) {}
    fn handle_event(&mut self, _event: E::EntityEvent) -> Vec<E> {
        vec![]
    }
}
