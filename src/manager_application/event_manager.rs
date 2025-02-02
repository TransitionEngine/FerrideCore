use std::fmt::Debug;

use winit::{dpi::PhysicalPosition, event::{ElementState, MouseButton, WindowEvent}, event_loop::ActiveEventLoop, window::WindowId};

use crate::graphics_provider::GraphicsProvider;

use super::WindowManager;

#[derive(Debug)]
pub struct MouseEvent {
    pub state: ElementState,
    pub button: MouseButton,
    pub position: PhysicalPosition<i32>
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
