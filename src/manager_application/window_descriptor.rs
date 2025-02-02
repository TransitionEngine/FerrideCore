use image::imageops::{resize, FilterType};
use std::fs;
pub mod winit_reexports {
    pub use winit::{
        dpi::{Position, Size},
        window::{Fullscreen, Icon, Theme, WindowButtons, WindowLevel},
    };
}
use winit::{
    event_loop::ActiveEventLoop,
    window::{CustomCursor, CustomCursorSource, WindowAttributes},
};
use winit_reexports::*;

#[derive(Clone, Debug)]
pub struct WindowDescriptor {
    attributes: WindowAttributes,
    cursor_path: Option<&'static str>,
    icon_path: Option<&'static str>,
}
impl WindowDescriptor {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_cursor(mut self, path: &'static str) -> Self {
        self.cursor_path = Some(path);
        self
    }

    pub fn with_icon(mut self, path: &'static str) -> Self {
        self.icon_path = Some(path);
        self
    }

    fn decode_icon(&self, path: &'static str) -> Icon {
        let bytes = fs::read(path).expect(&format!("Could not read icon file at '{}'", path));

        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory(&bytes)
                .expect(&format!("Could not parse icon file at '{}'", path))
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        Icon::from_rgba(icon_rgba, icon_width, icon_height)
            .expect(&format!("Could not make icon from file at '{}'", path))
    }

    fn decode_cursor(&self, path: &'static str) -> CustomCursorSource {
        let bytes = fs::read(path).expect(&format!("Could not read cursor file at '{}'", path));
        let img = image::load_from_memory(&bytes)
            .expect(&format!("Could not parse cursor file at '{}'", path))
            .into_rgba8();
        let img = resize(&img, 32, 32, FilterType::Gaussian);
        let samples = img.into_flat_samples();
        let (_, w, h) = samples.extents();
        let (w, h) = (w as u16, h as u16);
        CustomCursor::from_rgba(samples.samples, w, h, w / 4, 0)
            .expect(&format!("Could not make cursor from file at '{}'", path))
    }

    pub fn get_attributes(&self, event_loop: &ActiveEventLoop) -> WindowAttributes {
        let mut attributes = self.attributes.clone();
        if let Some(cursor_path) = self.cursor_path {
            let cursor_source = self.decode_cursor(cursor_path);
            attributes = attributes.with_cursor(event_loop.create_custom_cursor(cursor_source));
        }
        if let Some(icon_path) = self.icon_path {
            let icon = self.decode_icon(icon_path);
            attributes = attributes.with_window_icon(Some(icon));
        }
        attributes
    }
}
impl Default for WindowDescriptor {
    fn default() -> Self {
        Self {
            attributes: WindowAttributes::default(),
            cursor_path: None,
            icon_path: None,
        }
    }
}
///reimpl functions for WindowAttributes
impl WindowDescriptor {
    pub fn with_inner_size<S: Into<Size>>(mut self, size: S) -> Self {
        self.attributes = self.attributes.with_inner_size(size);
        self
    }
    pub fn with_min_inner_size<S: Into<Size>>(mut self, min_size: S) -> Self {
        self.attributes = self.attributes.with_min_inner_size(min_size);
        self
    }
    pub fn with_max_inner_size<S: Into<Size>>(mut self, max_size: S) -> Self {
        self.attributes = self.attributes.with_max_inner_size(max_size);
        self
    }
    pub fn with_position<P: Into<Position>>(mut self, position: P) -> Self {
        self.attributes = self.attributes.with_position(position);
        self
    }
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.attributes = self.attributes.with_resizable(resizable);
        self
    }
    pub fn with_enabled_buttons(mut self, buttons: WindowButtons) -> Self {
        self.attributes = self.attributes.with_enabled_buttons(buttons);
        self
    }
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.attributes = self.attributes.with_title(title);
        self
    }
    pub fn with_fullscreen(mut self, fullscreen: Option<Fullscreen>) -> Self {
        self.attributes = self.attributes.with_fullscreen(fullscreen);
        self
    }
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.attributes = self.attributes.with_maximized(maximized);
        self
    }
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.attributes = self.attributes.with_visible(visible);
        self
    }
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.attributes = self.attributes.with_transparent(transparent);
        self
    }
    pub fn with_blur(mut self, blur: bool) -> Self {
        self.attributes = self.attributes.with_blur(blur);
        self
    }
    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.attributes = self.attributes.with_decorations(decorations);
        self
    }
    pub fn with_window_level(mut self, level: WindowLevel) -> Self {
        self.attributes = self.attributes.with_window_level(level);
        self
    }
    pub fn with_theme(mut self, theme: Option<Theme>) -> Self {
        self.attributes = self.attributes.with_theme(theme);
        self
    }
    pub fn with_resize_increments<S: Into<Size>>(mut self, resize_increments: S) -> Self {
        self.attributes = self.attributes.with_resize_increments(resize_increments);
        self
    }
    pub fn with_content_protected(mut self, protected: bool) -> Self {
        self.attributes = self.attributes.with_content_protected(protected);
        self
    }
    pub fn with_active(mut self, active: bool) -> Self {
        self.attributes = self.attributes.with_active(active);
        self
    }
}
