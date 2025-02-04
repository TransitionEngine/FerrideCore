use std::fmt::Debug;

mod graphics_provider;
pub mod graphics {
    pub use super::graphics_provider::exports::*;
}

mod manager_application;
pub mod app {
    pub use super::manager_application::exports::*;
}

mod game;
pub mod game_engine {
    pub use super::game::exports::*;
    pub use super::game::{
        example, Game, State,
    };
}

pub trait Numeric: twod::Numeric + Into<f64> + Debug {}
impl <T: twod::Numeric + Into<f64> + Debug> Numeric for T {}

#[derive(Clone)]
pub struct Size<T: Numeric>(twod::Vector<T>);
impl <T: Numeric> Debug for Size<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Size{{{:?}, {:?}}}", self.width(), self.height())
    }
}
impl <T: Numeric> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self(twod::Vector::new(width, height))
    }
    pub fn width(&self) -> T {
        self.0.x
    }
    pub fn height(&self) -> T {
        self.0.y
    }
}
impl <T: Numeric> From<twod::Vector<T>> for Size<T> {
    fn from(value: twod::Vector<T>) -> Self {
        Self(value)
    }
}
impl <T: Numeric> From<Size<T>> for twod::Vector<T> {
    fn from(value: Size<T>) -> Self {
        value.0
    }
}
impl <T: Numeric> From<Size<T>> for winit::dpi::Size {
    fn from(value: Size<T>) -> Self {
        Self::Logical(winit::dpi::LogicalSize::new(value.width().into(), value.height().into()))
    }
}
impl <T: Numeric> From<winit::dpi::PhysicalSize<T>> for Size<T> {
    fn from(value: winit::dpi::PhysicalSize<T>) -> Self {
        Self(twod::Vector::new(value.width, value.height))
    }
}

#[derive(Clone)]
pub struct Position<T: Numeric>(twod::Vector<T>);
impl <T: Numeric> Position<T> {
    pub fn new(x: T, y: T) -> Self {
        Self(twod::Vector::new(x, y))
    }
    pub fn x(&self) -> T {
        self.0.x
    }
    pub fn y(&self) -> T {
        self.0.y
    }
}
impl <T: Numeric> Debug for Position<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Position({:?}, {:?})", self.x(), self.y())
    }
}
impl <T: Numeric> From<twod::Vector<T>> for Position<T> {
    fn from(value: twod::Vector<T>) -> Self {
        Self(value)
    }
}
impl <T: Numeric> From<Position<T>> for twod::Vector<T> {
    fn from(value: Position<T>) -> Self {
        value.0
    }
}
impl <T: Numeric> From<Position<T>> for winit::dpi::Position {
    fn from(value: Position<T>) -> Self {
        Self::Logical(winit::dpi::LogicalPosition::new(value.x().into(), value.y().into()))
    }
}

pub mod reexports {
    pub mod winit {
        pub use super::super::manager_application::winit_reexports::*;
    }
    pub mod wgpu {
        pub use wgpu::{vertex_attr_array, ShaderStages, VertexAttribute};
    }
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
