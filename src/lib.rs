use gtk::{IconTheme};

pub mod icon;
mod selector;
mod filter_widget;

pub use filter_widget::*;
pub use selector::*;

pub fn icon_theme() -> IconTheme {
    IconTheme::for_display(&gtk::gdk::Display::default().expect("Failed to get display"))
}
