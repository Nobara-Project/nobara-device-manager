mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct ColorBadge(ObjectSubclass<imp::ColorBadge>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ColorBadge {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
// ANCHOR_END: mod

impl Default for ColorBadge {
    fn default() -> Self {
        Self::new()
    }
}
