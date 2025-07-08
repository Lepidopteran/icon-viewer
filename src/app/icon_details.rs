use gtk::glib;

use nett_icon_viewer::icon::IconObject;

use super::data_row::DataRow;

const DEFAULT_ICON_SIZE: u32 = 128;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::{
        Allocation, CompositeTemplate, IconPaintable, Image, Label, ListItem, NoSelection,
        SignalListItemFactory, StringObject, TemplateChild, Widget,
        glib::{Properties, subclass::InitializingObject},
        prelude::*,
        subclass::prelude::*,
    };
    use nett_icon_viewer::icon_theme;

    use super::*;

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::IconDetails)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/icon_details.ui")]
    pub struct IconDetails {
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub container: TemplateChild<gtk::Box>,

        #[template_child]
        pub alias_button: TemplateChild<gtk::MenuButton>,

        #[template_child]
        pub alias_list: TemplateChild<gtk::ListView>,

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

        #[property(get, set = set_icon, nullable, construct)]
        icon: RefCell<Option<IconObject>>,

        #[property(get)]
        pub paintable: RefCell<Option<IconPaintable>>,

        #[property(get)]
        selection: RefCell<Option<NoSelection>>,
        bindings: RefCell<Vec<glib::Binding>>,
    }

    fn set_icon(imp: &IconDetails, icon: Option<IconObject>) {
        if let Some(icon) = icon.as_ref() {
            imp.bind_icon(icon);

            imp.paintable.borrow_mut().replace(icon_theme().lookup_icon(
                &icon.name(),
                &[],
                imp.icon_size.get() as i32,
                1,
                gtk::TextDirection::Ltr,
                gtk::IconLookupFlags::empty(),
            ));
            imp.obj().notify_paintable();

            imp.stack.set_visible_child_name("details");
        } else {
            imp.unbind_icon();
            imp.stack.set_visible_child_name("empty");
        }

        *imp.icon.borrow_mut() = icon;
        imp.obj().notify_icon();
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconDetails {
        const NAME: &'static str = "NettIconViewerIconDetails";
        type Type = super::IconDetails;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            DataRow::ensure_type();

            klass.set_css_name("icon-details");
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl IconDetails {
        fn bind_icon(&self, icon: &IconObject) {
            let mut bindings = self.bindings.borrow_mut();

            if let Some(selection) = self.selection.borrow().as_ref() {
                bindings.push(
                    icon.bind_property("aliases", selection, "model")
                        .transform_to(|_, v: Vec<String>| Some(gtk::StringList::from_iter(v)))
                        .sync_create()
                        .build(),
                );
            }
            let alias_button = &self.alias_button.get();
            let alias_binding = icon
                .bind_property("aliases", alias_button, "sensitive")
                .transform_to(|_, v: Vec<String>| Some(!v.is_empty()))
                .sync_create()
                .build();

            bindings.push(alias_binding);

            let label = &self.label.get();
            let label_binding = icon
                .bind_property("name", label, "label")
                .sync_create()
                .build();

            bindings.push(label_binding);

            let label_tooltip_binding = icon
                .bind_property("name", label, "tooltip-text")
                .sync_create()
                .build();

            bindings.push(label_tooltip_binding);

            let symbolic_row = &self.symbolic_row.get();
            let symbolic_row_binding = icon
                .bind_property("is-symbolic", symbolic_row, "value")
                .transform_to(|_, v: bool| Some(v.to_string().to_value()))
                .sync_create()
                .build();

            bindings.push(symbolic_row_binding);

            let tags_row = &self.tags_row.get();
            let tags_row_binding = icon
                .bind_property("tags", tags_row, "value")
                .transform_to(|_, v: Vec<String>| Some(v.join(", ").to_value()))
                .sync_create()
                .build();

            bindings.push(tags_row_binding);

            let path_row = &self.path_row.get();
            let path_row_binding = icon
                .bind_property("path", path_row, "value")
                .transform_to(|_, v: Option<String>| Some(v.unwrap_or_default().to_value()))
                .sync_create()
                .build();

            bindings.push(path_row_binding);

            let symlink_row = &self.symlink_row.get();
            let symlink_row_binding = icon
                .bind_property("is-symlink", symlink_row, "value")
                .transform_to(|_, v: bool| Some(v.to_string().to_value()))
                .sync_create()
                .build();

            bindings.push(symlink_row_binding);

            let symlink_path = &self.symlink_path_row.get();
            let symlink_path_binding = icon
                .bind_property("symlink-path", symlink_path, "value")
                .transform_to(|_, v: Option<String>| Some(v.unwrap_or_default().to_value()))
                .sync_create()
                .build();

            bindings.push(symlink_path_binding);
        }

        fn unbind_icon(&self) {
            for binding in self.bindings.borrow_mut().drain(..) {
                binding.unbind();
            }
        }

        #[template_callback]
        fn alias_activated(list: &gtk::ListView, index: u32) {
            let model = list.model().unwrap();
            let name = model
                .item(index)
                .as_ref()
                .unwrap()
                .downcast_ref::<StringObject>()
                .unwrap()
                .string();

            let clipboard = gtk::gdk::Display::default()
                .expect("Failed to get display")
                .clipboard();

            clipboard.set_text(&name);
            log::debug!("Copied \"{}\" to clipboard", name);
        }

        #[template_callback]
        fn copy_icon(&self) {
            let name = self.label.get().text();
            let clipboard = gtk::gdk::Display::default()
                .expect("Failed to get display")
                .clipboard();

            clipboard.set_text(&name);
            log::debug!("Copied \"{}\" to clipboard", name);
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconDetails {
        fn constructed(&self) {
            self.parent_constructed();

            let picture = self.picture.get();
            let outer = self.obj();
            let _ = outer
                .bind_property("icon-size", &picture, "width-request")
                .build();
            let _ = outer
                .bind_property("icon-size", &picture, "height-request")
                .build();
            let _ = outer
                .bind_property("paintable", &picture, "paintable")
                .build();

            let selection = NoSelection::new(None::<gtk::gio::ListModel>);

            let list = self.alias_list.get();

            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let label = Label::builder()
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .hexpand(true)
                    .xalign(0.0)
                    .build();

                let image = Image::builder()
                    .icon_name("edit-copy-symbolic")
                    .icon_size(gtk::IconSize::Normal)
                    .halign(gtk::Align::End)
                    .build();

                let container = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(12)
                    .margin_start(4)
                    .margin_end(4)
                    .margin_top(4)
                    .margin_bottom(4)
                    .hexpand(true)
                    .build();

                container.append(&label);
                container.append(&image);

                let list_item = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem");

                list_item.set_child(Some(&container));

                list_item
                    .property_expression("item")
                    .chain_property::<StringObject>("string")
                    .bind(&label, "label", Widget::NONE);
            });

            list.set_factory(Some(&factory));
            list.set_model(Some(&selection));
            self.selection.borrow_mut().replace(selection);
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
