mod graphics_provider;
pub mod graphics {
    pub use super::graphics_provider::{
        GraphicsProvider, Index, RenderSceneDescriptor, RenderSceneName, ShaderDescriptor,
        UniformBufferName, Vertex, Visibility, DEFAULT_TEXTURE,
    };
}

mod manager_application;
pub mod app {
    pub use super::manager_application::{
        ApplicationEvent, EventManager, IndexBuffer, ManagerApplication, VertexBuffer, write_regular_ngon_u16,
        WindowDescriptor, WindowManager, MouseEvent,
    };
}

mod game;
pub mod game_engine {
    pub use super::game::{
        example, static_camera, BoundingBox, CameraDescriptor, Direction, Entity, EntityName,
        EntityType, ExternalEvent, Game, RessourceDescriptor, RessourceDescriptorBuilder, Scene,
        SceneName, SpritePosition, SpriteSheet, SpriteSheetDimensions, SpriteSheetName, State,
        TextureCoordinates, VelocityController,
    };
}

pub mod reexports {
    pub mod winit {
        pub use super::super::manager_application::winit_reexports::*;
        pub use winit::dpi::PhysicalSize;
    }
    pub mod wgpu {
        pub use wgpu::{vertex_attr_array, ShaderStages, VertexAttribute};
    }
    // pub use wgpu;
    // pub use threed;
}

#[macro_export]
macro_rules! create_name_struct {
    ($name: ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $name(String);
        impl $name {
            #[allow(dead_code)]
            pub fn as_str<'a>(&'a self) -> &'a str {
                self.0.as_str()
            }
        }
        impl std::ops::Add<&str> for $name {
            type Output = Self;
            fn add(self, rhs: &str) -> Self::Output {
                (self.0 + rhs).into()
            }
        }
        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }
        impl From<String> for $name {
            fn from(value: String) -> Self {
                value.as_str().into()
            }
        }
        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                value.as_str().into()
            }
        }
    };
}
