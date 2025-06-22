use super::icon::*;
use gtk::prelude::*;
use gtk::{gio, glib};
use nett_icon_viewer::*;

mod imp {
    use gtk::{
        CompositeTemplate, Image, ListItem, SignalListItemFactory, SingleSelection,
        gio::ListStore, glib::subclass::prelude::*,
        subclass::prelude::*,
    };

    use super::*;

    #[derive(CompositeTemplate, Debug, Default)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/window.ui")]
    pub struct Window {
        #[template_child]
        pub view: TemplateChild<gtk::GridView>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "NettIconViewerWindow";
        type Type = super::Window;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let theme = icon_theme();
            let icons = theme
                .icon_names()
                .iter()
                .filter_map(|i| {
                    let paintable = theme.lookup_icon(
                        i,
                        &[],
                        0,
                        1,
                        gtk::TextDirection::Ltr,
                        gtk::IconLookupFlags::empty(),
                    );

                    if paintable.file().is_some() {
                        Some(IconData {
                            name: i.to_string(),
                            is_symbolic: paintable.is_symbolic(),
                            ..Default::default()
                        })
                    } else {
                        None
                    }
                })
                .map(IconObject::from)
                .collect::<Vec<_>>();

            let store = ListStore::new::<IconObject>();

            store.extend_from_slice(&icons);

            let factory = SignalListItemFactory::new();
            factory.connect_setup(move |_, list_item| {
                let label = Image::builder().icon_size(gtk::IconSize::Large).build();
                list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .set_child(Some(&label));
            });

            factory.connect_bind(move |_, list_item| {
                let icon = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .item()
                    .and_downcast::<IconObject>()
                    .expect("The item has to be an `String`.");

                let label = list_item
                    .downcast_ref::<ListItem>()
                    .expect("Needs to be ListItem")
                    .child()
                    .and_downcast::<Image>()
                    .expect("The child has to be a `Label`.");

                label.set_icon_name(Some(&icon.name()));
            });

            let selection = SingleSelection::builder().model(&store).build();

            self.view.set_model(Some(&selection));
            self.view.set_factory(Some(&factory));
        }
    }

    impl WidgetImpl for Window {}

    impl ApplicationWindowImpl for Window {}

    impl WindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &gtk::Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }
}
