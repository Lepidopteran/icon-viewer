use gtk::glib;
use gtk::glib::subclass::prelude::*;

use super::{
    CATEGORIES, FilterMode, FilterWidget,
    icon::{IconObject, IconWidget},
    icon_theme,
};

const DEFAULT_ICON_SIZE: u32 = 64;

mod imp {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use gtk::{
        Allocation, CompositeTemplate, ListItem, SignalListItemFactory, SingleSelection,
        TemplateChild,
        gio::{self, ListStore},
        glib::{Properties, subclass::InitializingObject},
        prelude::*,
        subclass::prelude::*,
    };

    use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

    use crate::icon::IconData;

    use super::*;

    type FilterFunction = Box<dyn Fn(&IconObject, &super::IconSelector) -> bool + 'static>;
    type SymlinkMessage = (usize, usize, Vec<(usize, String)>);

    #[derive(CompositeTemplate, Properties, Default)]
    #[properties(wrapper_type = super::IconSelector)]
    #[template(resource = "/codes/blaine/NettIconViewer/icon_selector.ui")]
    pub struct IconSelector {
        #[template_child]
        pub layout: TemplateChild<gtk::Box>,

        #[template_child]
        pub scroll: TemplateChild<gtk::ScrolledWindow>,

        #[template_child]
        pub view: TemplateChild<gtk::GridView>,

        #[template_child]
        pub count_label: TemplateChild<gtk::Label>,

        #[template_child]
        pub scale: TemplateChild<gtk::Scale>,

        #[template_child]
        pub search: TemplateChild<gtk::SearchEntry>,

        #[template_child]
        pub filter_widget: TemplateChild<FilterWidget>,

        #[template_child]
        pub status_revealer: TemplateChild<gtk::Revealer>,

        #[template_child]
        progress: TemplateChild<gtk::ProgressBar>,

        #[property(get, set = set_icon_size, construct, default = DEFAULT_ICON_SIZE)]
        pub icon_size: Cell<u32>,

        #[property(get, set)]
        pub selected: Cell<u32>,

        #[property(get, set)]
        pub copy_on_activate: Cell<bool>,

        #[property(get, set = set_include_tags_in_search, construct, default = true)]
        pub include_tags_in_search: Cell<bool>,

        #[property(get, set = set_included_tags, construct)]
        pub included_tags: RefCell<Vec<String>>,

        #[property(get)]
        pub num_items: Cell<u32>,

        displayed_icons: Rc<RefCell<Vec<IconWidget>>>,

        #[property(get, nullable)]
        icons: RefCell<Option<ListStore>>,
        sorter: gtk::CustomSorter,
        filter: gtk::CustomFilter,
        list: gtk::SortListModel,
    }

    fn set_icon_size(imp: &IconSelector, value: u32) {
        imp.icon_size.set(value);

        for cell in imp.displayed_icons.borrow().iter() {
            cell.set_icon_size(value);
        }

        imp.obj().notify_icon_size();
    }

    fn set_include_tags_in_search(imp: &IconSelector, value: bool) {
        imp.include_tags_in_search.set(value);
        imp.filter_changed();
        imp.obj().notify_include_tags_in_search();
    }

    fn set_included_tags(imp: &IconSelector, value: Vec<String>) {
        *imp.included_tags.borrow_mut() = value;
        imp.filter_changed();
        imp.obj().notify_included_tags();
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IconSelector {
        const NAME: &'static str = "NettIconViewerIconSelector";
        type Type = super::IconSelector;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("icon-selector");
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
            self.list.item(self.selected.get()).and_downcast()
        }

        fn icons(&self) -> ListStore {
            self.icons.borrow().clone().expect("Icons not set")
        }

        fn update_count_label(&self) {
            let count = self.num_items.get();
            let filtered_count = self.list.n_items();

            if count == filtered_count {
                self.count_label
                    .set_text(&format!("{} Icons", count.to_string().replace('0', "no")));
            } else {
                self.count_label.set_text(&format!(
                    "({:>padding$}/{}) Icons",
                    filtered_count.to_string(),
                    count,
                    padding = count.to_string().len()
                ));
            }
        }

        #[template_callback]
        fn filter_changed(&self) {
            self.filter.changed(gtk::FilterChange::Different);
        }

        #[template_callback]
        fn search_changed(&self) {
            self.filter_changed();
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

    fn handle_filter_pending(obj: &super::IconSelector) {
        let imp = obj.imp();
        imp.scroll.vadjustment().set_value(0.0);
        imp.update_count_label();
    }

    #[glib::derived_properties]
    impl ObjectImpl for IconSelector {
        fn constructed(&self) {
            self.parent_constructed();

            let theme = icon_theme();
            let icons = theme
                .icon_names()
                .iter()
                .map(|n| IconObject::new(n, self.icon_size.get()))
                .collect::<Vec<_>>();

            let data = icons
                .iter()
                .map(|icon| icon.data().clone())
                .collect::<Vec<_>>();

            let (symlinks, non_symlinks): (Vec<_>, Vec<_>) = data
                .iter()
                .cloned()
                .enumerate()
                .partition(|(_, data)| data.is_symlink);

            let non_symlinks_clone = non_symlinks.clone();

            let (alias_tx, alias_rx) = async_channel::bounded::<SymlinkMessage>(1);
            gio::spawn_blocking(move || {
                for (index, (icon_index, icon)) in non_symlinks_clone.iter().enumerate() {
                    let aliases: Vec<_> = symlinks
                        .iter()
                        .filter_map(|(i, s)| {
                            let IconData { symlink_path, .. } = s;

                            if let (Some(symlink_target), Some(path)) = (symlink_path, &icon.path) {
                                if symlink_target.is_absolute() && symlink_target == path
                                    || path.parent().unwrap().join(symlink_target) == *path
                                {
                                    Some((*i, s.name.clone()))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                        .collect();

                    alias_tx
                        .send_blocking((index, *icon_index, aliases))
                        .expect("Failed to send aliases");
                }
            });

            self.num_items.set(icons.len() as u32);
            self.obj().notify_num_items();

            let store = ListStore::new::<IconObject>();
            store.extend_from_slice(&icons);

            let status_revealer = self.status_revealer.get();
            let progress_bar = self.progress.get();
            let filter = self.filter.clone();
            let filter_widget = self.filter_widget.get();
            glib::spawn_future_local(async move {
                while let Ok((index, icon_index, aliases)) = alias_rx.recv().await {
                    let icon = icons.get(icon_index).unwrap();

                    for alias in aliases.clone() {
                        let alias_icon = icons.get(alias.0).unwrap();
                        alias_icon.set_symlink_target_index(icon_index as u32);
                    }

                    let aliases_names = aliases.iter().map(|(_, n)| n.to_string()).collect();

                    icon.add_aliases(aliases_names);

                    progress_bar.set_fraction((icon_index + 1) as f64 / icons.len() as f64);
                    status_revealer.set_reveal_child(index != non_symlinks.len() - 1);

                    if index == non_symlinks.len() - 1
                        && filter_widget.display_invalid_symlinks()
                    {
                        filter.changed(gtk::FilterChange::Different);
                    }
                }
            });

            self.icons.replace(Some(store));
            self.obj().notify_icons();

            let obj = self.obj().clone();
            self.filter.set_filter_func(move |item| {
                let search_text = obj.imp().search.text().to_string();
                let icon = item
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");

                let matcher = SkimMatcherV2::default();
                let mut matches = matcher.fuzzy_match(&icon.name(), &search_text).is_some()
                    || obj.include_tags_in_search()
                        && matcher
                            .fuzzy_match(&icon.tags().join(" "), &search_text)
                            .is_some();

                let filters: Vec<FilterFunction> = vec![
                    Box::new(|icon: &IconObject, selector: &super::IconSelector| {
                        let filter_widget = selector.imp().filter_widget.get();
                        match filter_widget.symlink_filter_mode() {
                            FilterMode::Is => icon.is_symlink(),
                            FilterMode::Not => {
                                filter_widget.display_invalid_symlinks() && !icon.has_target_index()
                                    || !icon.is_symlink()
                            }
                            FilterMode::Either => true,
                        }
                    }),
                    Box::new(|icon: &IconObject, selector: &super::IconSelector| {
                        match selector.imp().filter_widget.embedded_filter_mode() {
                            FilterMode::Is => icon.is_embedded(),
                            FilterMode::Not => !icon.is_embedded(),
                            FilterMode::Either => true,
                        }
                    }),
                    Box::new(|icon: &IconObject, selector: &super::IconSelector| {
                        match selector.imp().filter_widget.symbolic_filter_mode() {
                            FilterMode::Is => icon.is_symbolic(),
                            FilterMode::Not => !icon.is_symbolic(),
                            FilterMode::Either => true,
                        }
                    }),
                    Box::new(|icon: &IconObject, selector: &super::IconSelector| {
                        selector
                            .included_tags()
                            .iter()
                            .all(|tag| icon.tags().contains(tag))
                    }),
                    Box::new(|icon: &IconObject, selector: &super::IconSelector| {
                        let included_categories =
                            selector.imp().filter_widget.included_categories();

                        let tags: Vec<_> =
                            icon.tags().iter().map(|tag| tag.to_lowercase()).collect();

                        tags.iter().enumerate().any(|(index, tag)| {
                            included_categories
                                .iter()
                                .any(|c| tag.starts_with(c) && index != 0)
                        }) || included_categories.contains(&String::from("unknown"))
                            && tags
                                .iter()
                                .all(|tag| !CATEGORIES.iter().any(|c| tag.starts_with(c.1)))
                    }),
                ];

                matches &= filters.iter().all(|f| f(icon, &obj));

                matches
            });

            let filtered = gtk::FilterListModel::new(Some(self.icons()), Some(self.filter.clone()));
            filtered.set_incremental(true);

            let obj = self.obj().clone();
            filtered.connect_pending_notify(move |_| handle_filter_pending(&obj));

            let search_entry = self.search.get();
            self.sorter.set_sort_func(move |a, b| {
                let icon_a = a
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");
                let icon_b = b
                    .downcast_ref::<IconObject>()
                    .expect("Needs to be an `IconObject`.");

                let search_text = search_entry.text().to_string();
                let matcher = SkimMatcherV2::default();

                let score_a = matcher
                    .fuzzy_match(&icon_a.name(), &search_text)
                    .unwrap_or(0);
                let score_b = matcher
                    .fuzzy_match(&icon_b.name(), &search_text)
                    .unwrap_or(0);

                score_b
                    .cmp(&score_a)
                    .then_with(|| icon_a.name().cmp(&icon_b.name()))
                    .into()
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

            let search = self.search.get();

            let displayed_icons = self.displayed_icons.clone();
            let obj = self.obj().clone();
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

                cell.bind(&icon, search.text().as_ref(), obj.icon_size());

                displayed_icons.borrow_mut().push(cell);
            });

            let visible_cells = self.displayed_icons.clone();
            factory.connect_unbind(move |_, list_item| {
                let cell = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<IconWidget>()
                    .expect("The child has to be a `IconWidget`.");

                visible_cells.borrow_mut().retain(|c| *c != cell);

                cell.unbind();
            });

            let target = self.obj().clone();
            let selection = SingleSelection::builder().model(&sort).build();
            let _ = selection
                .bind_property("selected", &target, "selected")
                .bidirectional()
                .sync_create()
                .build();

            let scale = self.scale.get();

            for snap_scale in (32..256).step_by(32) {
                scale.add_mark(snap_scale as f64, gtk::PositionType::Top, None);
            }

            let icon_adjustment = scale.adjustment();
            let _ = target
                .bind_property("icon_size", &icon_adjustment, "value")
                .bidirectional()
                .sync_create()
                .build();

            self.list.set_model(Some(&sort));
            self.view.set_model(Some(&selection));
            self.view.set_factory(Some(&factory));
            self.update_count_label();
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

    pub fn connect_activate<F>(&self, f: F)
    where
        F: Fn(&gtk::GridView, u32) + 'static,
    {
        self.imp().view.connect_activate(f);
    }
}

impl Default for IconSelector {
    fn default() -> Self {
        Self::new()
    }
}
