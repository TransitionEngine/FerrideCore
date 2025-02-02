use std::fmt::Debug;

use winit::{
    event_loop::EventLoopProxy,
    window::{Window, WindowId},
};

pub struct WindowManager<E: 'static + Debug> {
    windows: Vec<Window>,
    event_loop: Option<EventLoopProxy<E>>,
}
impl<E: 'static + Debug> WindowManager<E> {
    pub fn set_event_loop(&mut self, event_loop: EventLoopProxy<E>) {
        self.event_loop = Some(event_loop);
    }

    fn get_event_loop(&self) -> &EventLoopProxy<E> {
        self.event_loop
            .as_ref()
            .expect("WindowManger must be initialized with '.set_event_loop' before sendind events")
    }

    pub fn send_event(&self, event: E) {
        self.get_event_loop()
            .send_event(event)
            .expect("The event loop has been closed. Cannot send an event");
    }

    pub fn create_event_loop_proxy(&self) -> EventLoopProxy<E> {
        self.get_event_loop().clone()
    }

    pub fn amount_windows(&self) -> usize {
        self.windows.len()
    }

    pub fn get_window(&self, id: &WindowId) -> Option<&Window> {
        self.windows.iter().find(|window| window.id() == *id)
    }

    pub fn remove_window(&mut self, id: &WindowId) {
        self.windows.retain(|window| window.id() != *id)
    }

    pub fn add_window(&mut self, window: Window) {
        self.windows.push(window);
    }
}
impl<E: 'static + Debug> Default for WindowManager<E> {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            event_loop: None,
        }
    }
}
