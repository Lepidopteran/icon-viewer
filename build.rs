fn main() {
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "nett-icon-viewer.gresource",
    );
}
