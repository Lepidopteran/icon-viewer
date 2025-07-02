use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use gtk::glib;
use gtk::glib::object::ObjectExt;
use gtk::glib::subclass::prelude::*;

use super::IconObject;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{Properties, subclass::InitializingObject};
    use gtk::prelude::*;
    use gtk::subclass::widget::WidgetImplExt;
    use gtk::subclass::widget::{
        CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
    };
    use gtk::{Allocation, CompositeTemplate, TemplateChild};

    use super::*;

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::IconWidget)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/icon_cell.ui")]
    pub struct IconWidget {
        #[template_child]
        pub container: TemplateChild<gtk::Grid>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub image: TemplateChild<gtk::Picture>,

        #[property(get, set)]
        pub icon_size: Cell<u32>,

        pub bindings: RefCell<Vec<glib::Binding>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconWidget {
        const NAME: &'static str = "NettIconViewerIconCell";
        type Type = super::IconWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("icon-cell");
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let target = self.image.clone();
            let outer = self.obj();
            let _ = outer
                .bind_property("icon-size", &target, "width-request")
                .build();
            let _ = outer
                .bind_property("icon-size", &target, "height-request")
                .build();
        }

        fn dispose(&self) {
            self.label.unparent();
            self.image.unparent();
            self.container.unparent();
        }
    }
    impl WidgetImpl for IconWidget {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.container.measure(orientation, for_size)
        }
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.container
                .size_allocate(&Allocation::new(0, 0, width, height), baseline);
        }
    }
}

glib::wrapper! {
    pub struct IconWidget(ObjectSubclass<imp::IconWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl IconWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn bind(&self, icon: &IconObject, search_text: &str, icon_size: u32) {
        let image = self.imp().image.clone();
        let label = self.imp().label.clone();
        let mut bindings = self.imp().bindings.borrow_mut();

        if icon_size != self.icon_size() {
            self.set_icon_size(icon_size);
        }

        let image_binding = icon
            .bind_property("paintable", &image, "paintable")
            .sync_create()
            .build();

        bindings.push(image_binding);

        let label_binding = icon
            .bind_property("name", &label, "label")
            .sync_create()
            .build();

        bindings.push(label_binding);

        let icon_size_binding = self
            .bind_property("icon-size", icon, "icon-size")
            .sync_create()
            .build();

        bindings.push(icon_size_binding);

        let text = label.text().to_string();
        let matcher = SkimMatcherV2::default();

        if search_text.is_empty() {
            label.set_markup(&glib::markup_escape_text(&text));
            return;
        }

        // Fuzzy match positions
        if let Some((_, indices)) = matcher.fuzzy_indices(&text, search_text) {
            let mut markup = String::new();
            for (i, c) in text.chars().enumerate() {
                if indices.contains(&i) {
                    markup.push_str(&format!("<span background='#99009955'><b>{}</b></span>", c));
                } else {
                    markup.push(c);
                }
            }
            label.set_markup(&markup);
        } else {
            label.set_markup(&glib::markup_escape_text(&text));
        }
    }

    pub fn unbind(&self) {
        for binding in self.imp().bindings.borrow_mut().drain(..) {
            binding.unbind();
        }

        self.imp().label.set_markup("");
    }
}

impl Default for IconWidget {
    fn default() -> Self {
        Self::new()
    }
}
