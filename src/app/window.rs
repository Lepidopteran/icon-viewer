use super::icon_details::IconDetails;
use gtk::prelude::*;
use gtk::{gio, glib};
use nett_icon_viewer::IconSelector;

mod imp {
    use std::cell::Cell;

    use gtk::{
        CompositeTemplate,
        glib::{Properties, subclass::prelude::*},
        subclass::prelude::*,
    };
    use nett_icon_viewer::icon::IconObject;
    use once_cell::sync::OnceCell;

    use super::*;

    #[derive(CompositeTemplate, Properties, Debug, Default)]
    #[template(resource = "/codes/blaine/nett-icon-viewer/window.ui")]
    #[properties(wrapper_type = super::Window)]
    pub struct Window {
        #[template_child]
        pub view: TemplateChild<IconSelector>,
        #[template_child]
        pub label: TemplateChild<gtk::Label>,
        #[template_child]
        pub icon_details: TemplateChild<IconDetails>,
        #[template_child]
        pub paned: TemplateChild<gtk::Paned>,

        #[property(get)]
        split_percentage: Cell<f64>,
        split_percentage_handler_id: OnceCell<glib::SignalHandlerId>,
    }

    impl Window {
        fn calculate_paned_position(&self) {
            let paned = self.paned.get();
            let percentage = self.split_percentage.get();
            let width = self.obj().width() as f64;

            self.block_handler();
            paned.set_position((width * percentage) as i32);

            self.unblock_handler();
        }

        fn block_handler(&self) {
            let handler_id = self.split_percentage_handler_id.get().unwrap();
            self.paned.get().block_signal(handler_id);
        }

        fn unblock_handler(&self) {
            let handler_id = self.split_percentage_handler_id.get().unwrap();
            self.paned.get().unblock_signal(handler_id);
        }

        fn unblock_handler_at_idle(&self) {
            let obj = self.obj().clone();

            glib::idle_add_local_once(move || {
                obj.imp().unblock_handler();
            });
        }
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

    #[glib::derived_properties]
    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj().clone();
            let paned = self.paned.get();
            let handler_id = paned.connect_position_notify(move |paned| {
                let percentage =
                    paned.position() as f64 / paned.size(gtk::Orientation::Horizontal) as f64;

                log::trace!("New split percentage: {}", percentage);

                obj.imp().split_percentage.set(percentage);
                obj.notify_split_percentage();
            });

            // NOTE: Block the signal before the initial position is set
            paned.block_signal(&handler_id);

            self.split_percentage_handler_id
                .set(handler_id)
                .expect("Failed to set handler");

            // TODO: Add ability to save on split percentage on exit.
            self.split_percentage.set(0.65);

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

    impl WidgetImpl for Window {
        fn map(&self) {
            self.parent_map();

            let obj = self.obj().clone();

            obj.connect_maximized_notify(|window| {
                let imp = window.imp();
                imp.block_handler();

                if window.is_maximized() {
                    imp.unblock_handler();
                } else {
                    imp.unblock_handler_at_idle();
                }
            });

            self.unblock_handler_at_idle();
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            self.calculate_paned_position();
        }
    }

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
        let window: Self = glib::Object::builder().property("application", app).build();

        window
    }
}
