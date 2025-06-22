use gtk::{IconTheme};

pub fn icon_theme() -> IconTheme {
    IconTheme::for_display(&gtk::gdk::Display::default().expect("Failed to get display"))
}
