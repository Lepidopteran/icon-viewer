use gtk::prelude::*;
use gtk::{gio, glib};
use nett_icon_viewer::{
    icon::{IconData, IconObject},
    icon_theme,
};

mod imp {
    use gtk::{
        CompositeTemplate, ListItem, SignalListItemFactory, SingleSelection, gio::ListStore,
        glib::subclass::prelude::*, subclass::prelude::*,
    };

    use crate::app::icon_cell::IconWidget;

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
                .map(|n| IconObject::new(n, 64))
                .collect::<Vec<_>>();

            let store = ListStore::new::<IconObject>();

            store.extend_from_slice(&icons);

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
