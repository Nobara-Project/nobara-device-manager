use crate::build_ui::colored_circle;
use glib::{clone, translate::ToGlibPtr, Properties};
use gtk::{ffi::gtk_widget_queue_draw, gdk, glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

// ANCHOR: custom_button
// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = colored_circle::ColoredCircle)]
pub struct ColoredCircle {
    #[property(get, set)]
    child: RefCell<Option<gtk::Widget>>,
    #[property(get, set)]
    color: RefCell<Option<gdk::RGBA>>,
}
// ANCHOR_END: custom_button

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ColoredCircle {
    const NAME: &'static str = "ColoredCircle";
    type Type = colored_circle::ColoredCircle;
    type ParentType = gtk::Widget;

    fn class_init(gtk_class: &mut Self::Class) {
        gtk_class.set_layout_manager_type::<gtk::BinLayout>();
    }
}

fn calculate_radius(w: f64, h: f64) -> f64 {
    std::cmp::min(w.round() as i64, (h - 1.0).round() as i64) as f64
}

fn redraw_widget(widget: &gtk::Widget) {
    unsafe {
        gtk_widget_queue_draw(widget.to_glib_full());
    }
}

// ANCHOR: object_impl
// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for ColoredCircle {
    fn constructed(&self) {
        #![allow(deprecated)]
        self.parent_constructed();

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        let obj = self.obj();

        let obj_clone0 = obj.clone();

        let draw_func =
            move |_da: &gtk::DrawingArea, cr: &gtk::cairo::Context, width: i32, height: i32| {
                let center_x = (width / 2) as f64;
                let center_y = (height / 2) as f64;
                let radius: f64 = calculate_radius(center_x, center_y);
                let color = obj_clone0
                    .color()
                    .unwrap_or(gdk::RGBA::new(60.0, 255.0, 0.0, 1.0));

                let delta = radius - 1.0;

                cr.save().unwrap();
                cr.move_to(center_x, center_y);
                cr.arc(
                    center_x,
                    center_y,
                    delta + 1.0,
                    1.5 * std::f64::consts::PI,
                    (2.5 * 2.0) * std::f64::consts::PI,
                );
                cr.set_source_color(&color);
                cr.fill().unwrap();

                let context = obj_clone0.style_context();
                context.save();
                // FIXME: Gtk4 has changes in the styles that need to be reviewed
                // For now we get the text color from the defaut context.
                cr.set_source_color(&context.color());
            };

        let child = gtk::DrawingArea::new();
        child.set_draw_func(draw_func);
        let child_widget = child.clone().upcast::<gtk::Widget>();
        //
        obj.connect_color_notify(clone!(
            #[strong]
            child_widget,
            move |_| {
                redraw_widget(&child_widget);
            }
        ));
        //
        child.set_parent(&*obj);
        *self.child.borrow_mut() = Some(child_widget);
    }

    fn dispose(&self) {
        // Child widgets need to be manually unparented in `dispose()`.
        if let Some(child) = self.child.borrow_mut().take() {
            child.unparent();
        }
    }
}
// Trait shared by all widgets
impl WidgetImpl for ColoredCircle {}
