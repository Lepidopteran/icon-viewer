use gtk::{IconPaintable, glib, prelude::*, subclass::prelude::*};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct IconData {
    pub name: String,
    pub categories: Vec<String>,
    pub paintable: Option<IconPaintable>,
    pub path: Option<PathBuf>,
    pub symbolic: bool,
    pub symlink: bool,
}

mod imp {
    use std::cell::RefCell;

    use gtk::glib::Properties;

    use super::*;

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::IconObject)]
    pub struct IconObject {
        #[property(name = "name", get, set, member = name, type = String)]
        #[property(name = "categories", get, set, member = categories, type = Vec<String>)]
        #[property(name = "symbolic", get, set, member = symbolic, type = bool)]
        #[property(name = "symlink", get, set, member = symlink, type = bool)]
        #[property(name = "paintable", get, set, member = paintable, type = Option<IconPaintable>)]
        #[property(name = "path",
            get = |o: &Self| o.data.borrow().path.as_ref().map(|p| p.display().to_string()),
            set = |o: &Self, v: Option<String>| o.data.borrow_mut().path = v.map(PathBuf::from),
            type = Option<String>
        )]
        pub data: RefCell<IconData>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconObject {
        const NAME: &'static str = "NettIconViewerIcon";
        type Type = super::IconObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconObject {}
}

glib::wrapper! {
    pub struct IconObject(ObjectSubclass<imp::IconObject>);
}

impl IconObject {
    pub fn new(
        name: &str,
        path: Option<PathBuf>,
        paintable: Option<IconPaintable>,
        categories: Vec<String>,
        symbolic: bool,
        symlink: bool,
    ) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("categories", categories)
            .property("symbolic", symbolic)
            .property("symlink", symlink)
            .property("path", path)
            .property("paintable", paintable)
            .build()
    }

    pub fn data(&self) -> IconData {
        self.imp().data.borrow().clone()
    }
}

impl From<IconData> for IconObject {
    fn from(data: IconData) -> Self {
        IconObject::new(
            &data.name,
            data.path,
            data.paintable,
            data.categories,
            data.symbolic,
            data.symlink,
        )
    }
}

impl From<IconObject> for IconData {
    fn from(icon: IconObject) -> Self {
        icon.data()
    }
}
