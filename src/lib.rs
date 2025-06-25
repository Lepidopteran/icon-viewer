use gtk::{IconTheme};

pub mod icon;
mod selector;
mod data_row;

pub use selector::*;
pub use data_row::*;

pub fn icon_theme() -> IconTheme {
    IconTheme::for_display(&gtk::gdk::Display::default().expect("Failed to get display"))
}
