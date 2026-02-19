mod config;

use adw::{prelude::*, *};
use cfhdb::{
    pci::{PreCheckedPciDevice, PreCheckedPciProfile},
    usb::{PreCheckedUsbDevice, PreCheckedUsbProfile},
};
use gdk::Display;
use gtk::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use std::{env, sync::Arc};
use sys_locale::get_locale;

use config::APP_ID;

pub enum ChannelMsg {
    OutputLine(String),
    SuccessMsg,
    UpdateMsg,
    SuccessMsgDeviceFetch(
        Option<Vec<(String, Vec<PreCheckedPciDevice>)>>,
        Option<Vec<(String, Vec<PreCheckedUsbDevice>)>>,
        PreCheckedDmiInfo,
        Option<Vec<(String, Vec<PreCheckedBtDevice>)>>,
        Vec<Arc<PreCheckedPciProfile>>,
        Vec<Arc<PreCheckedUsbProfile>>,
        Vec<Arc<PreCheckedDmiProfile>>,
        Vec<Arc<PreCheckedBtProfile>>,
    ),
    FailMsg,
}

// application crates
mod build_ui;
mod cfhdb;

use crate::{
    build_ui::build_ui,
    cfhdb::{
        bt::{PreCheckedBtDevice, PreCheckedBtProfile},
        dmi::{PreCheckedDmiInfo, PreCheckedDmiProfile},
    },
};

// Init translations for current crate.
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en_US");

/// main function
fn main() {
    let current_locale = get_locale().unwrap_or_else(|| String::from("en-US")).replace("-", "_");

    rust_i18n::set_locale(&current_locale);
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
