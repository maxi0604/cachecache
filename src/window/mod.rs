mod imp;

use std::path::PathBuf;

use gtk::{Application, glib::{self, object::ObjectBuilder}, gio};
use glib::Object;
use gtk::subclass::prelude::*;

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
    pub fn path_buf(&self) -> Option<PathBuf> {
        let result = self.imp()
            .path_buf
            .take();
        self.set_path_buf(result.clone());
        result
    }
    pub fn set_path_buf(&self, path_buf: Option<PathBuf>) {
        self.imp()
            .path_buf
            .set(path_buf);
    }
}
