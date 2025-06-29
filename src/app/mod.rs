use gtk::{Application, prelude::*};
mod data_row;
mod icon_details;
mod window;

pub const APP_ID: &str = "codes.blaine.nett-icon-viewer";

pub fn new() -> Application {
    let app = gtk::Application::new(Some(APP_ID), Default::default());
    app.connect_activate(build_ui);

    app
}

fn build_ui(app: &gtk::Application) {
    let window = window::Window::new(app);

    window.present();
}
