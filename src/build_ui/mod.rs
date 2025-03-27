mod colored_circle;
mod content;
mod loading;

use crate::build_ui::loading::loading_content;
use crate::cfhdb::pci::{get_pci_devices, get_pci_profiles_from_url};
use crate::cfhdb::usb::{get_usb_devices, get_usb_profiles_from_url};
use crate::config::{APP_GIT, APP_ICON, APP_ID, VERSION};
use crate::ChannelMsg;
use adw::prelude::*;
use adw::*;
use gtk::ffi::GtkWidget;
use gtk::glib::{clone, MainContext};
use gtk::Orientation::Vertical;
use gtk::{
    Align, Orientation, PolicyType, ScrolledWindow, SelectionMode, Stack, StackTransitionType,
    ToggleButton, Widget,
};
use libcfhdb::pci::CfhdbPciDevice;
use libcfhdb::usb::CfhdbUsbDevice;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn build_ui(app: &adw::Application) {
    // setup glib
    gtk::glib::set_prgname(Some(t!("app_name").to_string()));
    glib::set_application_name(&t!("app_name").to_string());
    //
    let glib_settings = gio::Settings::new(APP_ID);

    // create the main Application window
    let window = adw::ApplicationWindow::builder()
        .title(t!("application_name"))
        .application(app)
        .icon_name(APP_ICON)
        .width_request(400)
        .height_request(650)
        .default_width(glib_settings.int("window-width"))
        .default_height(glib_settings.int("window-height"))
        .startup_id(APP_ID)
        .build();

    if glib_settings.boolean("is-maximized") == true {
        window.maximize()
    }

    window.connect_close_request(move |window| {
        if let Some(application) = window.application() {
            save_window_size(&window, &glib_settings);
            application.remove_window(window);
        }
        std::process::exit(0);
    });

    loading_content(&window);

    // show the window
    window.present()
}

pub fn save_window_size(window: &adw::ApplicationWindow, glib_settings: &gio::Settings) {
    let size = window.default_size();

    let _ = glib_settings.set_int("window-width", size.0);
    let _ = glib_settings.set_int("window-height", size.1);
    let _ = glib_settings.set_boolean("is-maximized", window.is_maximized());
}
