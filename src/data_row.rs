use gtk::glib;
mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::Properties;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use super::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::DataRow)]
    pub struct DataRow {
        #[property(get, set)]
        title: RefCell<String>,

        #[property(get, set)]
        value: RefCell<String>,

        title_label: gtk::Label,
        value_label: gtk::Label,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DataRow {
        const NAME: &'static str = "NettIconViewerDataRow";
        type Type = super::DataRow;
        type ParentType = gtk::ListBoxRow;
    }

    #[glib::derived_properties]
    impl ObjectImpl for DataRow {
        fn constructed(&self) {
            self.parent_constructed();
            let title = &self.title_label;

            title.set_use_markup(true);
            title.set_xalign(0.0);
            title.set_opacity(0.5);
            title.set_halign(gtk::Align::Start);

            let text = &self.value_label;
            text.set_xalign(0.0);
            text.set_halign(gtk::Align::Start);

            let obj = self.obj();

            let _ = obj.bind_property("title", title, "label").build();
            let _ = obj.bind_property("value", text, "label").build();

            let child = gtk::Box::new(gtk::Orientation::Vertical, 0);

            child.append(title);
            child.append(text);

            obj.set_child(Some(&child));
        }
    }

    impl WidgetImpl for DataRow {}
    impl ListBoxRowImpl for DataRow {}
}

glib::wrapper! {
    pub struct DataRow(ObjectSubclass<imp::DataRow>)
        @extends gtk::ListBoxRow, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl DataRow {
    pub fn new(title: &str, value: &str) -> Self {
        glib::Object::builder()
            .property("title", title)
            .property("value", value)
            .build()
    }
}

impl Default for DataRow {
    fn default() -> Self {
        Self::new("No title", "No text")
    }
}
