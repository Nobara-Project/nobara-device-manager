mod color_badge;
mod colored_circle;
mod content;
mod loading;

use crate::{
    build_ui::loading::loading_content,
    config::{APP_ICON, APP_ID},
};
use adw::{glib::clone, prelude::*, *};

pub fn build_ui(app: &adw::Application) {
    // setup glib
    gtk::glib::set_prgname(Some(t!("application_name").to_string()));
    glib::set_application_name(&t!("application_name").to_string());
    //
    let glib_settings = gio::Settings::new(APP_ID);

    // = Global Menu =

    let app_menu = gio::Menu::new();
    app.set_menubar(Some(&app_menu));

    // == File menu ==

    let file_menu_item = gio::MenuItem::new(Some(&t!("file_menu_item_label")), None);

    let file_menu = gio::Menu::new();
    file_menu_item.set_submenu(Some(&file_menu));

    file_menu.append(Some(&t!("file_menu_item_app_quit_label")), Some("app.quit"));

    let quit_action = gio::SimpleAction::new("quit", None);
    app.add_action(&quit_action);

    // == View menu ==

    let view_menu_item = gio::MenuItem::new(Some(&t!("edit_menu_view_label")), None);

    let view_menu = gio::Menu::new();
    view_menu_item.set_submenu(Some(&view_menu));

    view_menu.append(
        Some(&t!("view_menu_item_app_showallprofiles")),
        Some("app.showallprofiles"),
    );
    let showallprofiles_action = gio::SimpleAction::new("showallprofiles", None);
    app.add_action(&showallprofiles_action);

    // == Help menu ==

    let help_menu_item = gio::MenuItem::new(Some(&t!("help_menu_item_label")), None);

    let help_menu = gio::Menu::new();
    help_menu_item.set_submenu(Some(&help_menu));

    help_menu.append(
        Some(&t!("help_menu_item_app_about_label")),
        Some("app.about"),
    );

    let about_action = gio::SimpleAction::new("about", None);
    app.add_action(&about_action);

    //

    app_menu.append_item(&file_menu_item);
    app_menu.append_item(&view_menu_item);
    app_menu.append_item(&help_menu_item);

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

    loading_content(&window, &about_action, &showallprofiles_action);

    // show the window
    window.present();

    quit_action.connect_activate(clone!(
        #[strong]
        window,
        move |_, _| {
            window.destroy();
        }
    ));
}

pub fn save_window_size(window: &adw::ApplicationWindow, glib_settings: &gio::Settings) {
    let size = window.default_size();

    let _ = glib_settings.set_int("window-width", size.0);
    let _ = glib_settings.set_int("window-height", size.1);
    let _ = glib_settings.set_boolean("is-maximized", window.is_maximized());
}
