use std::path::PathBuf;

use gtk::glib;
use gtk::subclass::prelude::*;

#[derive(Default)]
pub struct CacheCacheWindow {
    pub path_buf: std::cell::Cell<Option<PathBuf>>
}

#[glib::object_subclass]
impl ObjectSubclass for CacheCacheWindow {
    const NAME: &'static str = "CacheCacheWindow";
    type Type = super::CacheCacheWindow;
    type ParentType = gtk::ApplicationWindow;
}

impl ObjectImpl for CacheCacheWindow {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();
        obj.set_path_buf(None);
    }
}

impl WidgetImpl for CacheCacheWindow {}

impl WindowImpl for CacheCacheWindow {}

impl ApplicationWindowImpl for CacheCacheWindow {} 
