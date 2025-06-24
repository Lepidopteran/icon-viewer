use gtk::glib;
use gtk::glib::subclass::prelude::*;

use super::{
    icon::{IconObject, IconWidget},
    icon_theme,
};

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::gio::ListStore;
    use gtk::glib::{Properties, subclass::InitializingObject};
    use gtk::subclass::widget::WidgetImplExt;
    use gtk::subclass::widget::{
        CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
    };
    use gtk::{Allocation, CompositeTemplate, TemplateChild};
    use gtk::{ListItem, SignalListItemFactory, SingleSelection, prelude::*};

    use super::*;

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::IconSelector)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/icon_selector.ui")]
    pub struct IconSelector {
        #[template_child]
        pub layout: TemplateChild<gtk::Box>,
        #[template_child]
        pub view: TemplateChild<gtk::GridView>,

        #[property(get, set)]
        pub icon_size: Cell<u32>,

        model: RefCell<Option<gtk::SortListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconSelector {
        const NAME: &'static str = "NettIconViewerIconSelector";
        type Type = super::IconSelector;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconSelector {
        fn constructed(&self) {
            self.parent_constructed();

            let theme = icon_theme();
            let icons = theme
                .icon_names()
                .iter()
                .map(|n| IconObject::new(n, 64))
                .collect::<Vec<_>>();

            let store = ListStore::new::<IconObject>();

            store.extend_from_slice(&icons);
            let filter = gtk::FilterListModel::new(Some(store), None::<gtk::Filter>);

            let sorter = gtk::CustomSorter::new(|a, b| {
                let icon_a = a
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");
                let icon_b = b
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");

                gtk::Ordering::from(icon_a.name().cmp(&icon_b.name()))
            });

            let sort = gtk::SortListModel::new(Some(filter), Some(sorter));
            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let cell = IconWidget::new();
                list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&cell));
            });

            factory.connect_bind(move |_, list_item| {
                let icon = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .item()
                    .and_downcast::<IconObject>()
                    .expect("The item has to be an `String`.");

                let cell = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<IconWidget>()
                    .expect("The child has to be a `IconWidget`.");

                cell.bind(&icon);
            });

            factory.connect_unbind(move |_, list_item| {
                let cell = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<IconWidget>()
                    .expect("The child has to be a `IconWidget`.");

                cell.unbind();
            });

            let selection = SingleSelection::builder().model(&sort).build();

            self.model.replace(Some(sort));
            self.view.set_model(Some(&selection));
            self.view.set_factory(Some(&factory));
        }

        fn dispose(&self) {
            self.layout.unparent();
        }
    }
    impl WidgetImpl for IconSelector {
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
    pub struct IconSelector(ObjectSubclass<imp::IconSelector>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl IconSelector {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for IconSelector {
    fn default() -> Self {
        Self::new()
    }
}
