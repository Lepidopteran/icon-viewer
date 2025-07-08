use gtk::glib;
use gtk::glib::subclass::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default, glib::Enum)]
#[enum_type(name = "NettIconViewerFilterDisplayMode")]
pub enum FilterMode {
    #[default]
    Is,
    Not,
    Either,
}

use super::CATEGORIES;

mod imp {
    use std::cell::RefCell;
    use std::collections::HashSet;

    use gtk::glib::{Properties, subclass::InitializingObject};
    use gtk::prelude::*;
    use gtk::subclass::widget::{CompositeTemplateCallbacksClass, WidgetImplExt};
    use gtk::subclass::widget::{
        CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
    };
    use gtk::{Allocation, CompositeTemplate, TemplateChild};

    use super::*;

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::FilterWidget)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/icon_selector_filters.ui")]
    pub struct FilterWidget {
        #[template_child]
        layout: TemplateChild<gtk::Box>,

        #[template_child]
        category_box: TemplateChild<gtk::Box>,

        #[template_child]
        symbolic_check: TemplateChild<gtk::CheckButton>,

        #[template_child]
        symlink_check: TemplateChild<gtk::CheckButton>,

        #[property(get, set = set_symlink_filter_mode, construct, builder(FilterMode::Not))]
        pub symlink_filter_mode: RefCell<FilterMode>,

        #[property(get, set = set_symbolic_filter_mode, construct, builder(FilterMode::Either))]
        pub symbolic_filter_mode: RefCell<FilterMode>,

        #[property(get, set = set_included_categories)]
        pub included_categories: RefCell<Vec<String>>,
    }

    fn set_symlink_filter_mode(imp: &FilterWidget, mode: FilterMode) {
        map_filter_mode_to_check(&imp.symlink_check, &mode);

        *imp.symlink_filter_mode.borrow_mut() = mode;
        imp.obj().notify_symlink_filter_mode();
    }

    fn set_symbolic_filter_mode(imp: &FilterWidget, mode: FilterMode) {
        map_filter_mode_to_check(&imp.symbolic_check, &mode);

        *imp.symbolic_filter_mode.borrow_mut() = mode;
        imp.obj().notify_symbolic_filter_mode();
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
        fn symbolic_toggled(&self) {
            let obj = self.obj();

            let new_mode = match obj.symbolic_filter_mode() {
                FilterMode::Is => FilterMode::Not,
                FilterMode::Not => FilterMode::Either,
                FilterMode::Either => FilterMode::Is,
            };

            obj.set_symbolic_filter_mode(new_mode);
        }

        #[template_callback]
        fn symlink_toggled(&self) {
            let obj = self.obj();

            let new_mode = match obj.symlink_filter_mode() {
                FilterMode::Is => FilterMode::Not,
                FilterMode::Not => FilterMode::Either,
                FilterMode::Either => FilterMode::Is,
            };

            obj.set_symlink_filter_mode(new_mode);
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

    fn map_filter_mode_to_check(check: &gtk::CheckButton, mode: &FilterMode) {
        check.set_active(*mode == FilterMode::Is);
        check.set_inconsistent(*mode == FilterMode::Either);
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
            let obj = self.obj().clone();

            map_filter_mode_to_check(&self.symbolic_check, &obj.symbolic_filter_mode());
            map_filter_mode_to_check(&self.symlink_check, &obj.symlink_filter_mode());

            for (name, value) in CATEGORIES.iter().chain(&[("Unknown", "unknown")]) {
                let check = gtk::CheckButton::builder()
                    .label(*name)
                    .active(true)
                    .build();
                self.included_categories
                    .borrow_mut()
                    .push(value.to_string());

                let obj = self.obj().clone();
                obj.bind_property("included-categories", &check, "active")
                    .transform_to(move |_, v: Vec<String>| Some(v.contains(&value.to_string())))
                    .build();

                let obj = self.obj().clone();
                check.connect_toggled(move |check| {
                    if check.is_active() {
                        obj.imp().add_category(value);
                    } else {
                        obj.imp().remove_category(value);
                    }
                });

                self.category_box.append(&check);
            }
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
