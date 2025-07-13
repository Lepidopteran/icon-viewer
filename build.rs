use image::load_from_memory_with_format;
use resvg::{tiny_skia, usvg};

const ICON_BUILD_DIR: &str = "resources/build";

fn main() {
    println!("cargo::rerun-if-changed=data");
    println!("cargo::rerun-if-changed=resources/icon.svg");

    let app_name = env!("CARGO_PKG_NAME");
    let app_icon = include_str!("resources/icon.svg");
    let tree = {
        let mut opt = usvg::Options::default();
        opt.fontdb_mut().load_system_fonts();

        usvg::Tree::from_str(app_icon, &opt).unwrap()
    };

    let pixmap_size = tree.size().to_int_size().scale_to_width(1024).unwrap();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();

    let scale_x = pixmap_size.width() as f32 / tree.size().width();
    let scale_y = pixmap_size.height() as f32 / tree.size().height();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );

    let data = pixmap.encode_png().unwrap();
    if !std::path::Path::new(ICON_BUILD_DIR).exists() {
        std::fs::create_dir(ICON_BUILD_DIR).unwrap();
    }

    let img =
        load_from_memory_with_format(&data, image::ImageFormat::Png).expect("Failed to load image");

    std::fs::write(
        format!("{}/{}-1024x1024.png", ICON_BUILD_DIR, app_name),
        data,
    )
    .unwrap();

    for size in [32, 64, 128, 256, 512] {
        let file_name = match size {
            256 => "128x128@2x.png".to_string(),
            _ => format!("{}-{}x{}.png", app_name, size, size),
        };

        let resized = img.resize(size, size, image::imageops::FilterType::Nearest);

        resized
            .save(format!("{}/{}", ICON_BUILD_DIR, file_name))
            .unwrap();
    }

    std::fs::write(format!("{}/{}.svg", ICON_BUILD_DIR, app_name), app_icon).unwrap();
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "NettIconViewer.gresource",
    );
}
