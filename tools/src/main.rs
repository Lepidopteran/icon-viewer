use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};

use image::load_from_memory_with_format;
use resvg::{tiny_skia, usvg};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let now = std::time::Instant::now();
    let out_dir = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("resources/build"));

    println!("Generating icons in \"{}\"...", out_dir.display());

    let out_filename = "codes.blaine.NettIconViewer";

    let app_icon = include_str!("../../resources/icon.svg");
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

    let img =
        load_from_memory_with_format(&data, image::ImageFormat::Png).expect("Failed to load image");

    let dir = out_dir.join("1024x1024").join("apps");
    if !dir.exists() {
        create_dir_all(&dir).unwrap();
    }

    write(dir.join(format!("{out_filename}.png")), &data).unwrap();

    for size in [16, 24, 32, 48, 64, 128, 256, 512] {
        let dir_name = match size {
            256 => "128x128@2x".to_string(),
            _ => format!("{size}x{size}"),
        };

        let dir = out_dir.join(dir_name).join("apps");
        if !dir.exists() {
            create_dir_all(&dir).unwrap();
        }

        let resized = img.resize(size, size, image::imageops::FilterType::Triangle);
        resized
            .save(dir.join(format!("{out_filename}.png")))
            .unwrap();
    }

    let scale_dir = out_dir.join("scalable").join("apps");
    if !scale_dir.exists() {
        create_dir_all(&scale_dir).unwrap();
    }

    write(scale_dir.join(format!("{out_filename}.svg")), app_icon).unwrap();

    println!("Generated icons in {}ms", now.elapsed().as_millis());
}
