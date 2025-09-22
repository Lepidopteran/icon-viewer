use gtk::{Application, CssProvider, gdk::Display, prelude::*};
mod data_row;
mod icon_details;
mod window;

pub const APP_ID: &str = "codes.blaine.NettIconViewer";

pub fn new() -> Application {
    let app = gtk::Application::new(Some(APP_ID), Default::default());
    app.connect_startup(|_| {
        load_css();
    });
    app.connect_activate(build_ui);

    app
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("../../data/css/application.css"));

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &gtk::Application) {
    let window = window::Window::new(app);

    window.present();
}
