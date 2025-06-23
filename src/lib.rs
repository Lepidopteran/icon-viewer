use gtk::{IconTheme};

pub mod icon;
pub mod category;
mod selector;

pub use selector::*;

pub fn icon_theme() -> IconTheme {
    IconTheme::for_display(&gtk::gdk::Display::default().expect("Failed to get display"))
}
