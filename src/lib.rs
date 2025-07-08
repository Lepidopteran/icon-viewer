use gtk::{IconTheme};

pub mod icon;
mod selector;
mod filter_widget;

pub use filter_widget::*;
pub use selector::*;

pub const CATEGORIES: &[(&str, &str)] = &[
    ("Actions", "actions"),
    ("Animations", "animations"),
    ("Applications", "apps"),
    ("Categories", "categories"),
    ("Devices", "devices"),
    ("Emblems", "emblems"),
    ("Emotes", "emotes"),
    ("International", "intl"),
    ("MimeTypes", "mimetypes"),
    ("Places", "places"),
    ("Status", "status"),
];

pub fn icon_theme() -> IconTheme {
    IconTheme::for_display(&gtk::gdk::Display::default().expect("Failed to get display"))
}
