use gtk::prelude::*;
use gtk::{gio, glib};
use nett_icon_viewer::{IconDetails, IconSelector};

mod imp {
    use gtk::{CompositeTemplate, glib::subclass::prelude::*, subclass::prelude::*};
    use nett_icon_viewer::icon::IconObject;

    use super::*;

    #[derive(CompositeTemplate, Debug, Default)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/window.ui")]
    pub struct Window {
        #[template_child]
        pub view: TemplateChild<IconSelector>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub icon_details: TemplateChild<IconDetails>,
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

            let details = self.icon_details.get();
            self.view.connect_activate(move |view, index| {
                if let Some(icon) = view
                    .model()
                    .and_then(|m| m.item(index).and_downcast::<IconObject>())
                {
                    details.set_icon_name(icon.name());
                }
            });
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
