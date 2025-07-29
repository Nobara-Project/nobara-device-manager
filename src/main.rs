mod config;

use adw::{prelude::*, *};
use cfhdb::{
    pci::{PreCheckedPciDevice, PreCheckedPciProfile},
    usb::{PreCheckedUsbDevice, PreCheckedUsbProfile},
};
use gdk::Display;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use std::{env, sync::Arc};

use config::APP_ID;

pub enum ChannelMsg {
    OutputLine(String),
    SuccessMsg,
    UpdateMsg,
    SuccessMsgDeviceFetch(
        Vec<(String, Vec<PreCheckedPciDevice>)>,
        Vec<(String, Vec<PreCheckedUsbDevice>)>,
        Vec<Arc<PreCheckedPciProfile>>,
        Vec<Arc<PreCheckedUsbProfile>>,
    ),
    FailMsg,
}

// application crates
mod build_ui;
mod cfhdb;

use crate::build_ui::build_ui;

// Init translations for current crate.
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en_US");

/// main function
fn main() {
    // Get the current locale from the LANG environment variable
    let current_locale = match env::var_os("LANG") {
        Some(v) => v.into_string().unwrap(),
        None => String::from("en_US.UTF-8"), // Default to English if LANG is not set
    };

    // Strip the .UTF-8 suffix if present
    let locale = current_locale
        .strip_suffix(".UTF-8")
        .unwrap_or(&current_locale);

    // Set the locale for translations
    rust_i18n::set_locale(locale);

    // Print the current locale for debugging - convert to string first
    let current_locale_str = rust_i18n::locale().to_string();
    println!("Current locale: {}", current_locale_str);

    let application = adw::Application::new(Some(APP_ID), Default::default());
    application.connect_startup(|app| {
        // The CSS "magic" happens here.
        let provider = CssProvider::new();
        provider.load_from_string(include_str!("style.css"));
        // We give the CssProvided to the default screen so the CSS rules we added
        // can be applied to our window.
        gtk::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        app.connect_activate(build_ui);
    });

    application.run();
}
