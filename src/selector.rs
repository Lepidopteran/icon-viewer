use gtk::glib;
use gtk::glib::subclass::prelude::*;

use super::{
    icon::{IconObject, IconWidget},
    icon_theme,
};

mod imp {
    use std::cell::{Cell, RefCell};

    use gtk::{
        Allocation, CompositeTemplate, ListItem, SignalListItemFactory, SingleSelection,
        TemplateChild,
        gio::ListStore,
        glib::{Properties, subclass::InitializingObject},
        prelude::*,
        subclass::widget::{
            CompositeTemplateClass, CompositeTemplateInitializingExt, WidgetClassExt, WidgetImpl,
            WidgetImplExt,
        },
    };

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

        #[property(get, set)]
        pub selected: Cell<u32>,

        #[property(get, set)]
        pub copy_on_activate: Cell<bool>,

        model: RefCell<Option<gtk::SortListModel>>,
    }

    impl IconSelector {
        pub fn get_selected_icon(&self) -> Option<IconObject> {
            self.model
                .borrow()
                .as_ref()
                .and_then(|m| m.item(self.selected.get()))
                .and_then(|i| i.downcast_ref::<IconObject>().cloned())
        }
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

            let target = self.obj().clone();
            let selection = SingleSelection::builder().model(&sort).build();
            let _ = selection
                .bind_property("selected", &target, "selected")
                .bidirectional()
                .sync_create()
                .build();

            let selector = self.obj().clone();
            self.view.connect_activate(move |_, _| {
                if selector.copy_on_activate() {
                    let icon = selector.selected_icon();
                    if let Some(icon) = icon {
                        let clipboard = gtk::gdk::Display::default()
                            .expect("Failed to get display")
                            .clipboard();

                        clipboard.set_text(&icon.name());
                        log::debug!("Copied \"{}\" to clipboard", icon.name());
                    }
                }
            });

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

    pub fn selected_icon(&self) -> Option<IconObject> {
        self.imp().get_selected_icon()
    }
}

impl Default for IconSelector {
    fn default() -> Self {
        Self::new()
    }
}
