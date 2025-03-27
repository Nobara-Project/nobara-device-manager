mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper! {
    pub struct ColoredCircle(ObjectSubclass<imp::ColoredCircle>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl ColoredCircle {
    pub fn new() -> Self {
        Object::builder().build()
    }
}
// ANCHOR_END: mod

impl Default for ColoredCircle {
    fn default() -> Self {
        Self::new()
    }
}
