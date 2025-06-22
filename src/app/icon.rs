use gtk::{glib, prelude::*, subclass::prelude::*};

#[derive(Debug, Default, Clone)]
pub struct IconData {
    pub name: String,
    pub categories: Vec<String>,
    pub is_symbolic: bool,
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
        #[property(name = "is-symbolic", get, set, member = is_symbolic, type = bool)]
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
    pub fn new(name: &str, categories: Vec<String>, is_symbolic: bool) -> Self {
        glib::Object::builder()
            .property("name", name)
            .property("categories", categories)
            .property("is-symbolic", is_symbolic)
            .build()
    }

    pub fn data(&self) -> IconData {
        self.imp().data.borrow().clone()
    }
}

impl From<IconData> for IconObject {
    fn from(data: IconData) -> Self {
        IconObject::new(&data.name, data.categories, data.is_symbolic)
    }
}

impl From<IconObject> for IconData {
    fn from(icon: IconObject) -> Self {
        icon.data()
    }
}
