use std::{error::Error, fmt::Display, time::Duration};

use threed::Vector;
use winit::{
    dpi::PhysicalSize,
    event::KeyEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::game_engine::BoundingBox;

use super::{
    entity::{EntityName, EntityType},
    Direction, Entity, ExternalEvent, VelocityController,
};

pub fn static_camera(view_size: PhysicalSize<f32>) -> [[f32; 2]; 3] {
    [
        [2.0 / view_size.width, 0.0],
        [0.0, 2.0 / view_size.height],
        [0.0, 0.0],
    ]
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view: [[f32; 2]; 3],
}
impl From<&Camera> for CameraUniform {
    fn from(camera: &Camera) -> Self {
        let x = camera.position.x + camera.offset_position.x;
        let y = camera.position.y + camera.offset_position.y;
        let c = Self {
            view: [
                [2.0 / camera.view_size.width, 0.0],
                [0.0, 2.0 / camera.view_size.height],
                [
                    -2.0 * x / camera.view_size.width,
                    -2.0 * y / camera.view_size.height,
                ],
            ],
        };
        c
    }
}

#[derive(Clone)]
pub struct CameraDescriptor {
    pub view_size: PhysicalSize<f32>,
    pub speed: f32,
    pub acceleration_steps: u32,
    pub target_entity: EntityName,
    ///Entity whose bounding box will restrict the movement of the camera
    ///The cameras bounding box described by position and view_size will stay inside this
    ///bounding box
    pub bound_entity: Option<EntityName>,
    pub max_offset_position: f32,
}
impl From<&CameraDescriptor> for Camera {
    fn from(descriptor: &CameraDescriptor) -> Self {
        Self::new(descriptor)
    }
}

#[derive(Debug)]
pub enum CameraUpdateFailed {
    NoTargetEntity(EntityName),
    NOBoundEntity(EntityName),
}
impl Display for CameraUpdateFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CameraUpdateFailed::NoTargetEntity(name) => {
                write!(
                    f,
                    "No target entity with name: {:?} a found for camera",
                    name
                )
            }
            CameraUpdateFailed::NOBoundEntity(name) => {
                write!(f, "No bound entity with name: {:?} found for camera", name)
            }
        }
    }
}
impl Error for CameraUpdateFailed {}

pub struct Camera {
    position: Vector<f32>,
    offset_position: Vector<f32>,
    max_offset: f32,
    decceleration_factor: f32,
    velocity: VelocityController,
    view_size: PhysicalSize<f32>,
    target_entity: EntityName,
    bound_entity: Option<EntityName>,
}
impl Camera {
    fn new(descriptor: &CameraDescriptor) -> Self {
        Self {
            position: Vector::new(0.0, 0.0, 0.0),
            offset_position: Vector::new(0.0, 0.0, 0.0),
            max_offset: descriptor.max_offset_position,
            decceleration_factor: 1.0 - 1.0 / descriptor.acceleration_steps as f32,
            velocity: VelocityController::new(
                descriptor.speed / descriptor.acceleration_steps as f32,
            ),
            view_size: descriptor.view_size,
            bound_entity: descriptor.bound_entity.clone(),
            target_entity: descriptor.target_entity.clone(),
        }
    }

    pub fn reset_offset(&mut self) {
        self.velocity.stop_movement();
        self.offset_position = Vector::scalar(0.0);
    }

    pub fn update<T: EntityType, E: ExternalEvent>(
        &mut self,
        entities: Vec<&Box<dyn Entity<T, E>>>,
        _delta_t: &Duration,
    ) -> Result<(), CameraUpdateFailed> {
        let target_entity = match entities
            .iter()
            .find(|entity| entity.name() == &self.target_entity)
        {
            Some(entity) => entity,
            None => {
                return Err(CameraUpdateFailed::NoTargetEntity(
                    self.target_entity.clone(),
                ))
            }
        };
        let velocity = self.velocity.get_velocity();
        if velocity.x.abs() <= 1e-4 {
            self.offset_position.x *= self.decceleration_factor;
        }
        if velocity.y.abs() <= 1e-4 {
            self.offset_position.y *= self.decceleration_factor;
        }
        self.offset_position += velocity;
        if self.offset_position.magnitude_squared() >= self.max_offset.powi(2) {
            self.offset_position = self.offset_position.normalize() * self.max_offset;
        }
        self.position = target_entity.position();
        if let Some(bound_entity) = &self.bound_entity {
            let bound_entity = match entities.iter().find(|entity| entity.name() == bound_entity) {
                Some(entity) => entity,
                None => return Err(CameraUpdateFailed::NOBoundEntity(bound_entity.clone())),
            };
            match bound_entity.bounding_box().clamp_box_inside(&BoundingBox {
                anchor: &self.position + &self.offset_position,
                size: self.view_size,
            }) {
                None => {}
                Some(new_offset) => self.position = new_offset - &self.offset_position,
            };
        }
        Ok(())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(bytemuck::cast_slice(&CameraUniform::from(self).view));
        v
    }

    pub fn handle_key_input(&mut self, input: &KeyEvent) {
        if input.state == winit::event::ElementState::Released {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, false);
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, false);
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, false);
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, false);
                }
                _ => {}
            }
        } else if input.state == winit::event::ElementState::Pressed {
            match input.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    self.velocity.set_direction(Direction::Up, true);
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    self.velocity.set_direction(Direction::Left, true);
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    self.velocity.set_direction(Direction::Right, true);
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    self.velocity.set_direction(Direction::Down, true);
                }
                _ => {}
            }
        }
    }
}
