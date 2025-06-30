use gtk::glib;
use gtk::glib::object::ObjectExt;
use gtk::glib::subclass::prelude::*;

mod imp {
    use std::cell::{Cell, RefCell};
    use std::collections::HashSet;

    use gtk::glib::{Properties, subclass::InitializingObject};
    use gtk::subclass::widget::{CompositeTemplateCallbacksClass, WidgetImplExt};
    use gtk::subclass::widget::{
        CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
    };
    use gtk::{Allocation, CompositeTemplate, TemplateChild};
    use gtk::{StringObject, prelude::*};

    use super::*;

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::FilterWidget)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/icon_selector_filters.ui")]
    pub struct FilterWidget {
        #[template_child]
        pub categories: TemplateChild<gtk::StringList>,

        #[template_child]
        pub layout: TemplateChild<gtk::Box>,

        #[template_child]
        pub category_list: TemplateChild<gtk::ListBox>,

        #[property(get, set = set_included_categories)]
        pub included_categories: RefCell<Vec<String>>,
    }

    fn set_included_categories(imp: &FilterWidget, included_categories: Vec<String>) {
        let included_categories_set: HashSet<_> = HashSet::from_iter(included_categories);

        imp.included_categories
            .replace(included_categories_set.into_iter().collect());

        imp.obj().notify_included_categories();
    }

    #[gtk::template_callbacks]
    impl FilterWidget {
        #[template_callback]
        fn category_list_row_activated(_: &gtk::ListBox, row: &gtk::ListBoxRow) {
            let child = row.child().expect("Failed to get child");
            let check = child
                .downcast_ref::<gtk::CheckButton>()
                .expect("Failed to get check button");

            check.set_active(!check.is_active());
        }

        fn remove_category(&self, category: &str) {
            let mut included_categories = self.included_categories.borrow_mut().clone();
            included_categories.retain(|c| c != category);

            set_included_categories(self, included_categories);
        }

        fn add_category(&self, category: &str) {
            let mut included_categories = self.included_categories.borrow_mut().clone();
            included_categories.push(category.to_string());

            set_included_categories(self, included_categories);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterWidget {
        const NAME: &'static str = "NettIconViewerSelectorFilters";
        type Type = super::FilterWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("icon-selector-filters");
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FilterWidget {
        fn constructed(&self) {
            self.parent_constructed();

            let categories = self.categories.get();

            let obj = self.obj().clone();
            self.category_list
                .bind_model(Some(&categories), move |item| {
                    let obj = obj.clone();
                    let category = item
                        .downcast_ref::<StringObject>()
                        .map(|s| s.string().to_string())
                        .unwrap();

                    obj.imp().add_category(&category);

                    let check = gtk::CheckButton::builder().label(&category).build();
                    let category_clone = category.clone();
                    obj.bind_property("included-categories", &check, "active").transform_to(move |_, v: Vec<String>| {
                        Some(v.contains(&category_clone))
                    }).build();

                    let category_clone = category.clone();
                    check.connect_toggled(move |check| {
                        if check.is_active() {
                            obj.imp().add_category(&category_clone);
                        } else {
                            obj.imp().remove_category(&category_clone);
                        }
                    });

                    check.into()
                });
        }

        fn dispose(&self) {
            self.layout.unparent();
        }
    }
    impl WidgetImpl for FilterWidget {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.layout.measure(orientation, for_size)
        }
        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.layout
                .size_allocate(&Allocation::new(0, 0, width, height), baseline);
        }
    }
}

glib::wrapper! {
    pub struct FilterWidget(ObjectSubclass<imp::FilterWidget>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl FilterWidget {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for FilterWidget {
    fn default() -> Self {
        Self::new()
    }
}
