use gtk::prelude::*;
use gtk::{gio, glib};

mod app;

fn main() -> glib::ExitCode {
    dotenvy::dotenv().ok();
    env_logger::init();
    color_eyre::install().ok();

    gtk::init().expect("Failed to initialize GTK");
    gio::resources_register_include!("NettIconViewer.gresource")
        .expect("Failed to register resources");

    let app = app::new();

    app.run()
}
