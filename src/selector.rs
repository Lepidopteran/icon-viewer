use gtk::glib;
use gtk::glib::subclass::prelude::*;

use super::{
    icon::{IconObject, IconWidget},
    icon_theme,
};

mod imp {
    use std::cell::Cell;

    use gtk::{
        Allocation, CompositeTemplate, ListItem, SignalListItemFactory, SingleSelection,
        TemplateChild,
        gio::ListStore,
        glib::{Properties, subclass::InitializingObject},
        prelude::*,
        subclass::prelude::*,
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
        #[template_child]
        pub search: TemplateChild<gtk::SearchEntry>,

        #[property(get, set)]
        pub icon_size: Cell<u32>,

        #[property(get, set)]
        pub selected: Cell<u32>,

        #[property(get, set)]
        pub copy_on_activate: Cell<bool>,


        sorter: gtk::CustomSorter,
        filter: gtk::CustomFilter,
        model: gtk::SortListModel,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconSelector {
        const NAME: &'static str = "NettIconViewerIconSelector";
        type Type = super::IconSelector;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[gtk::template_callbacks]
    impl IconSelector {
        pub fn get_selected_icon(&self) -> Option<IconObject> {
            self.model.item(self.selected.get()).and_downcast()
        }

        #[template_callback]
        fn filter_changed(&self) {
            self.filter.changed(gtk::FilterChange::Different);
        }

        #[template_callback]
        fn view_activate(&self) {
            if self.copy_on_activate.get() {
                let icon = self.get_selected_icon();
                if let Some(icon) = icon {
                    let clipboard = gtk::gdk::Display::default()
                        .expect("Failed to get display")
                        .clipboard();

                    clipboard.set_text(&icon.name());
                    log::debug!("Copied \"{}\" to clipboard", icon.name());
                }
            }
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

            let search_entry = self.search.clone();
            self.filter.set_filter_func(move |item| {
                let search_text = search_entry.text().to_string();
                let icon = item
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");

                if search_text.is_empty() {
                    true
                } else {
                    icon.name().starts_with(&search_text)
                }
            });

            let filtered = gtk::FilterListModel::new(Some(store), Some(self.filter.clone()));

            self.sorter.set_sort_func(|a, b| {
                let icon_a = a
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");
                let icon_b = b
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");

                gtk::Ordering::from(icon_a.name().cmp(&icon_b.name()))
            });

            let sort = gtk::SortListModel::new(Some(filtered), Some(self.sorter.clone()));
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

            self.model.set_model(Some(&sort));
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
