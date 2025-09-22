
fn main() {
    println!("cargo::rerun-if-changed=data");
    println!("cargo::rerun-if-changed=resources/icon.svg");

    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "NettIconViewer.gresource",
    );
}
