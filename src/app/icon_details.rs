use gtk::glib;

use nett_icon_viewer::{
    icon::{IconObject},
};

use super::data_row::DataRow;

const DEFAULT_ICON_SIZE: u32 = 128;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::{
        Allocation, CompositeTemplate, TemplateChild,
        glib::{Properties, subclass::InitializingObject},
        prelude::*,
        subclass::prelude::*,
    };

    use super::{*};

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::IconDetails)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/icon_details.ui")]
    pub struct IconDetails {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub container: TemplateChild<gtk::Box>,

        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,

        #[template_child]
        pub label: TemplateChild<gtk::Label>,

        #[template_child]
        pub list: TemplateChild<gtk::ListBox>,

        #[template_child]
        pub tags_row: TemplateChild<DataRow>,

        #[template_child]
        pub path_row: TemplateChild<DataRow>,

        #[template_child]
        pub symbolic_row: TemplateChild<DataRow>,

        #[template_child]
        pub symlink_row: TemplateChild<DataRow>,

        #[template_child]
        pub symlink_path_row: TemplateChild<DataRow>,

        #[property(get, set, construct, default = DEFAULT_ICON_SIZE)]
        pub icon_size: Cell<u32>,

        #[property(get, set)]
        pub icon_name: RefCell<String>,

        icon: RefCell<Option<IconObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconDetails {
        const NAME: &'static str = "NettIconViewerIconDetails";
        type Type = super::IconDetails;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            DataRow::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl IconDetails {}

    #[glib::derived_properties]
    impl ObjectImpl for IconDetails {
        fn constructed(&self) {
            self.parent_constructed();

            let picture = self.picture.get();
            let label = self.label.get();
            let outer = self.obj();
            let _ = outer
                .bind_property("icon-size", &picture, "width-request")
                .build();
            let _ = outer
                .bind_property("icon-size", &picture, "height-request")
                .build();

            let _ = outer.bind_property("icon-name", &label, "label").build();
            let _ = outer
                .bind_property("icon-name", &label, "tooltip-text")
                .build();

            outer.connect_icon_name_notify(move |icon_details| {
                let icon_name = icon_details.icon_name();
                let icon_size = icon_details.icon_size();
                let inner = icon_details.imp();
                let mut current_icon = inner.icon.borrow_mut();

                if current_icon.is_none() && icon_name.is_empty() {
                    return;
                }

                if let Some(icon) = current_icon.as_ref() {
                    if icon_name.is_empty() {
                        inner.stack.set_visible_child_name("empty");
                        inner.icon.replace(None);
                        return;
                    }
                    icon.set_name(icon_name);
                } else {
                    let icon = IconObject::new(&icon_name, icon_size);
                    let _ = icon
                        .bind_property("paintable", &picture, "paintable")
                        .sync_create()
                        .build();

                    let _ = icon_details
                        .bind_property("icon-size", &icon, "icon-size")
                        .sync_create()
                        .build();

                    let symbolic = &inner.symbolic_row.get();
                    let _ = icon
                        .bind_property("symbolic", symbolic, "value")
                        .transform_to(|_, v: bool| Some(v.to_string().to_value()))
                        .sync_create()
                        .build();

                    let tags = &inner.tags_row.get();
                    let _ = icon
                        .bind_property("categories", tags, "value")
                        .transform_to(|_, v: Vec<String>| Some(v.join(", ").to_value()))
                        .sync_create()
                        .build();

                    let path = &inner.path_row.get();
                    let _ = icon
                        .bind_property("path", path, "value")
                        .transform_to(|_, v: String| Some(v.to_value()))
                        .sync_create()
                        .build();

                    let symlink = &inner.symlink_row.get();
                    let _ = icon
                        .bind_property("symlink", symlink, "value")
                        .transform_to(|_, v: bool| Some(v.to_string().to_value()))
                        .sync_create()
                        .build();

                    let symlink_path = &inner.symlink_path_row.get();
                    let _ = icon
                        .bind_property("symlink-path", symlink_path, "value")
                        .transform_to(|_, v: Option<String>| Some(v.unwrap_or_default().to_value()))
                        .sync_create()
                        .build();

                    current_icon.replace(icon);
                    inner.stack.set_visible_child_name("details");
                }
            });
        }

        fn dispose(&self) {
            self.stack.unparent();
        }
    }

    impl WidgetImpl for IconDetails {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.stack.measure(orientation, for_size)
        }
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.stack
                .size_allocate(&Allocation::new(0, 0, width, height), baseline);
        }
    }
}

glib::wrapper! {
    pub struct IconDetails(ObjectSubclass<imp::IconDetails>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl IconDetails {
    pub fn new(icon_name: &str, icon_size: u32) -> Self {
        glib::Object::builder()
            .property("icon-name", icon_name)
            .property("icon-size", icon_size)
            .build()
    }
}

impl Default for IconDetails {
    fn default() -> Self {
        Self::new("", DEFAULT_ICON_SIZE)
    }
}
