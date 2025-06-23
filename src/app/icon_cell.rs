use gtk::glib::subclass::prelude::*;
use gtk::{IconPaintable, glib};

use crate::app::icon::IconObject;

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::glib::{subclass::InitializingObject, Properties};
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
        pub image: TemplateChild<gtk::Image>,

        #[property(get, set)]
        pub icon_size: Cell<u32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconWidget {
        const NAME: &'static str = "NettIconViewerIconCell";
        type Type = super::IconWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
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
            let _ = outer.bind_property("icon-size", &target, "width-request").build();
            let _ = outer.bind_property("icon-size", &target, "height-request").build();
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

    pub fn bind_data(&self, icon: &IconObject) {
        self.imp().label.set_text(&icon.name());
        self.imp().image.set_paintable(icon.paintable().as_ref());
    }
}

impl Default for IconWidget {
    fn default() -> Self {
        Self::new()
    }
}
