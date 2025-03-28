use crate::build_ui::color_badge;
use glib::{clone, translate::ToGlibPtr, Properties};
use gtk::cairo;
use gtk::gdk;
use gtk::glib;
use gtk::pango;
use gtk::SizeGroup;
use gtk::{ffi::gtk_widget_queue_draw, prelude::*, subclass::prelude::*, Align};
use std::cell::RefCell;

// ANCHOR: custom_button
// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = color_badge::ColorBadge)]
pub struct ColorBadge {
    #[property(get, set)]
    child: RefCell<Option<gtk::Widget>>,
    #[property(get, set)]
    color: RefCell<Option<gdk::RGBA>>,
    #[property(get, set)]
    label0: RefCell<String>,
    #[property(get, set)]
    label1: RefCell<String>,
    #[property(get, set)]
    css_style: RefCell<String>,
    #[property(get, set)]
    group_size0: RefCell<Option<SizeGroup>>,
    #[property(get, set)]
    group_size1: RefCell<Option<SizeGroup>>,
    #[property(get, set)]
    theme_changed_action: RefCell<Option<gtk::gio::SimpleAction>>,
}
// ANCHOR_END: custom_button

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ColorBadge {
    const NAME: &'static str = "ColorBadge";
    type Type = color_badge::ColorBadge;
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
impl ObjectImpl for ColorBadge {
    fn constructed(&self) {
        self.parent_constructed();

        // Bind label to number
        // `SYNC_CREATE` ensures that the label will be immediately set
        let obj = self.obj();

        let obj_clone0 = obj.clone();

        let badge_box = gtk::Box::builder().build();

        let label0 = gtk::Label::builder()
            .margin_start(5)
            .margin_end(5)
            .margin_bottom(1)
            .margin_top(1)
            .valign(Align::Center)
            .halign(Align::Center)
            .hexpand(true)
            .vexpand(true)
            .build();
        obj.connect_group_size0_notify(clone!(
            #[strong]
            obj,
            #[strong]
            label0,
            move |_| {
                match obj.group_size0() {
                    Some(t) => {
                        t.add_widget(&label0);
                    }
                    None => {}
                }
            }
        ));

        let _ = label0
            .bind_property("label", &obj_clone0, "label0")
            .sync_create()
            .bidirectional()
            .build();

        let label_seprator = gtk::Separator::builder().build();

        let label1 = gtk::Label::builder()
            .margin_start(3)
            .margin_end(0)
            .margin_bottom(1)
            .margin_top(1)
            .valign(Align::Center)
            .halign(Align::Center)
            .hexpand(true)
            .vexpand(true)
            .build();
        obj.connect_group_size1_notify(clone!(
            #[strong]
            obj,
            #[strong]
            label1,
            move |_| {
                match obj.group_size1() {
                    Some(t) => {
                        t.add_widget(&label1);
                    }
                    None => {}
                }
            }
        ));

        let _ = label1
            .bind_property("label", &obj_clone0, "label1")
            .sync_create()
            .bidirectional()
            .build();

        obj.connect_css_style_notify(clone!(
            #[strong]
            obj,
            #[strong]
            label1,
            move |_| {
                label1.add_css_class(&obj.css_style());
            }
        ));

        #[allow(deprecated)]
        let color = label1
            .style_context()
            .lookup_color("accent_bg_color")
            .unwrap();
        if (color.red() * 0.299 + color.green() * 0.587 + color.blue() * 0.114) > 170.0 {
            label1.remove_css_class("white-color-text");
            label1.add_css_class("black-color-text");
        } else {
            label1.remove_css_class("black-color-text");
            label1.add_css_class("white-color-text");
        }

        obj.connect_theme_changed_action_notify(clone!(
            #[strong]
            obj,
            #[strong]
            label1,
            move |_| {
                match obj.theme_changed_action() {
                    Some(t) => {
                        t.connect_activate(clone!(
                            #[strong]
                            label1,
                            move |_, _| {
                                #[allow(deprecated)]
                                let color = label1
                                    .style_context()
                                    .lookup_color("accent_bg_color")
                                    .unwrap();
                                if (color.red() * 0.299
                                    + color.green() * 0.587
                                    + color.blue() * 0.114)
                                    > 170.0
                                {
                                    label1.remove_css_class("white-color-text");
                                    label1.add_css_class("black-color-text");
                                } else {
                                    label1.remove_css_class("black-color-text");
                                    label1.add_css_class("white-color-text");
                                }
                            }
                        ));
                    }
                    None => {}
                }
            }
        ));

        badge_box.append(&label0);
        badge_box.append(&label_seprator);
        badge_box.append(&label1);

        let boxedlist = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .halign(Align::Center)
            .valign(Align::End)
            .margin_start(5)
            .margin_end(5)
            .margin_bottom(5)
            .margin_top(5)
            .build();

        boxedlist.add_css_class("boxed-list");
        boxedlist.append(&badge_box);

        let child = boxedlist;
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
impl WidgetImpl for ColorBadge {}
