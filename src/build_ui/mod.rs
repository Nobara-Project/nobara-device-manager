mod color_badge;
mod colored_circle;
mod content;
mod loading;

use crate::{
    build_ui::loading::loading_content,
    config::{APP_ICON, APP_ID},
};
use adw::{prelude::*, *};

pub fn build_ui(app: &adw::Application) {
    // setup glib
    gtk::glib::set_prgname(Some(t!("application_name").to_string()));
    glib::set_application_name(&t!("application_name").to_string());
    //
    let glib_settings = gio::Settings::new(APP_ID);

    // create the main Application window
    let window = adw::ApplicationWindow::builder()
        .title(t!("application_name"))
        .application(app)
        .icon_name(APP_ICON)
        .width_request(600)
        .height_request(750)
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
