use gtk::{IconPaintable, glib, prelude::*, subclass::prelude::*};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct IconData {
    pub name: String,
    pub categories: Vec<String>,
    pub path: Option<PathBuf>,
    pub symbolic: bool,
    pub symlink: bool,
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::Properties;

    use crate::icon_theme;

    use super::*;

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::IconObject)]
    pub struct IconObject {
        #[property(name = "name", get, set, member = name, type = String)]
        #[property(name = "categories", get, member = categories, type = Vec<String>)]
        #[property(name = "symbolic", get, member = symbolic, type = bool)]
        #[property(name = "symlink", get, member = symlink, type = bool)]
        #[property(name = "path",
            get = |o: &Self| o.data.borrow().path.as_ref().map(|p| p.display().to_string()),
            type = Option<String>
        )]
        pub data: RefCell<IconData>,

        #[property(get, set)]
        pub paintable: RefCell<Option<IconPaintable>>,
        #[property(get, set)]
        pub icon_size: Cell<u32>,
    }

    impl IconObject {
        pub fn setup_events(&self) {
            let outer = self.obj();
            outer.connect_icon_size_notify(move |outer| {
                let inner = outer.imp();
                inner.render_icon(false);
            });

            outer.connect_name_notify(move |outer| {
                let inner = outer.imp();
                inner.render_icon(true);
            });
        }

        fn render_icon(&self, name_changed: bool) {
            let mut data = self.data.borrow_mut();
            let size = self.icon_size.get();
            let paintable = icon_theme().lookup_icon(
                &data.name,
                &[],
                size as i32,
                1,
                gtk::TextDirection::Ltr,
                gtk::IconLookupFlags::empty(),
            );

            if name_changed {
                if let Some(path) = paintable.file().and_then(|f| f.path()) {
                    let is_symlink = std::fs::symlink_metadata(&path)
                        .map(|m| m.file_type().is_symlink())
                        .unwrap_or(false);

                    data.symlink = is_symlink;
                    data.path = Some(path);
                }
                data.path = paintable.file().and_then(|f| f.path());
                data.symbolic = paintable.is_symbolic();
            }

            self.obj().set_paintable(paintable);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconObject {
        const NAME: &'static str = "NettIconViewerIcon";
        type Type = super::IconObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconObject {
        fn constructed(&self) {
            self.parent_constructed();
            self.render_icon(true);
            self.setup_events();
        }
    }
}

glib::wrapper! {
    pub struct IconObject(ObjectSubclass<imp::IconObject>);
}

impl IconObject {
    pub fn new(name: &str, icon_size: u32) -> Self {
        let icon: Self = glib::Object::builder()
            .property("name", name)
            .property("icon-size", icon_size)
            .build();

        icon
    }

    pub fn data(&self) -> IconData {
        self.imp().data.borrow().clone()
    }
}

impl From<IconObject> for IconData {
    fn from(icon: IconObject) -> Self {
        icon.data()
    }
}
