use std::fmt::Debug;

use winit::{event::{ElementState, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, window::WindowId};
pub mod winit_reexports {
    pub use winit::event;
    pub use winit::event::{ElementState, MouseButton};
}

pub mod exports {
    pub use super::EventManager;
    pub use super::MouseEvent;
}

use crate::{graphics_provider::GraphicsProvider, Position};

use super::window_manager::WindowManager;

#[derive(Debug)]
pub struct MouseEvent {
    pub state: ElementState,
    pub button: MouseButton,
    pub position: Position<i32>
}

pub trait EventManager<E: 'static + Debug> {
    /// Handles window events in a WindowManager. Return `false` to prevent default behavior of the
    /// WindowManager. Default behavior is closing, resizing and rendering the window and toggling fullscreen on F11
    fn window_event(
        &mut self,
        window_manager: &mut WindowManager<E>,
        event_loop: &ActiveEventLoop,
        id: &WindowId,
        event: &WindowEvent,
    ) -> bool
    where
        Self: Sized;
    fn user_event(
        &mut self,
        _window_manager: &mut WindowManager<E>,
        _graphics_provider: &mut GraphicsProvider,
        _event_loop: &ActiveEventLoop,
        _event: E,
    ) where
        Self: Sized,
    {
    }
}
