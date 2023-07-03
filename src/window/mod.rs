mod imp;

use gtk::{Application, glib::{self, object::ObjectBuilder}, gio};
use glib::Object;

glib::wrapper! {
    pub struct CacheCacheWindow(ObjectSubclass<imp::CacheCacheWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl CacheCacheWindow {
    pub fn new(app: &Application) -> Self {
        Self::builder()
            .property("application", app)
            .build()
    }
    pub fn builder() -> ObjectBuilder<'static, Self> {
        Object::builder()
            .property("default-height", 400)
            .property("default-width", 500)
            .property("title", "CacheCache")
    }
}
