use crate::build_ui::color_badge::ColorBadge;
use crate::build_ui::loading::run_in_lock_script;
use crate::cfhdb::usb::{get_usb_devices, get_usb_profiles_from_url};
use crate::config::{APP_GIT, APP_ICON, APP_ID, VERSION};
use crate::ChannelMsg;
use adw::prelude::*;
use adw::*;
use gtk::ffi::GtkWidget;
use gtk::gdk::RGBA;
use gtk::glib::{clone, MainContext};
use gtk::pango::Color;
use gtk::Orientation::Vertical;
use gtk::{
    Align, Orientation, PolicyType, ScrolledWindow, SelectionMode, Stack, StackTransitionType,
    ToggleButton, Widget,
};
use libcfhdb::usb::{CfhdbUsbDevice, CfhdbUsbProfile};
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use users::get_current_username;

use super::colored_circle::{self, ColoredCircle};
use super::{error_dialog, exec_duct_with_live_channel_stdout};

pub fn create_usb_class(
    window: &ApplicationWindow,
    devices: &Vec<CfhdbUsbDevice>,
    class: &str,
    theme_changed_action: &gio::SimpleAction,
    update_device_status_action: &gio::SimpleAction,
) -> ScrolledWindow {
    let devices_list_row = gtk::ListBox::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .selection_mode(SelectionMode::Browse)
        .vexpand(true)
        .hexpand(true)
        .build();
    devices_list_row.add_css_class("boxed-list");
    //
    let devices_navigation_page_toolbar = adw::ToolbarView::builder()
        .content(&devices_list_row)
        .build();
    devices_navigation_page_toolbar.add_top_bar(
        &adw::HeaderBar::builder()
            .show_end_title_buttons(false)
            .show_start_title_buttons(false)
            .build(),
    );
    let devices_navigation_page = adw::NavigationPage::builder()
        .title(class)
        .child(&devices_navigation_page_toolbar)
        .build();
    //
    let navigation_view = adw::NavigationView::builder().build();
    navigation_view.add(&devices_navigation_page);
    let scroll = gtk::ScrolledWindow::builder()
        .max_content_width(650)
        .min_content_width(300)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&navigation_view)
        .build();
    //
    for device in devices {
        let device_status_indicator = colored_circle::ColoredCircle::new();
        device_status_indicator.set_width_request(15);
        device_status_indicator.set_height_request(15);
        let device_title = format!("{} - {}", &device.manufacturer_string_index, &device.product_string_index);
        let device_navigation_page_toolbar = adw::ToolbarView::builder()
            .content(&usb_device_page(
                &window,
                &device,
                &theme_changed_action,
                &update_device_status_action,
                &device_status_indicator,
            ))
            .build();
        device_navigation_page_toolbar.add_top_bar(
            &adw::HeaderBar::builder()
                .show_end_title_buttons(false)
                .show_start_title_buttons(false)
                .build(),
        );
        let device_navigation_page = adw::NavigationPage::builder()
            .title(&device_title)
            .child(&device_navigation_page_toolbar)
            .build();
        navigation_view.add(&device_navigation_page);
        let action_row = adw::ActionRow::builder()
            .title(&device_title)
            .subtitle(&device.sysfs_busid)
            .activatable(true)
            .build();
        action_row.connect_activated(clone!(
            #[weak]
            navigation_view,
            #[weak]
            device_navigation_page,
            move |_| {
                navigation_view.push(&device_navigation_page);
            }
        ));
        action_row.add_suffix(&device_status_indicator);
        devices_list_row.append(&action_row);
    }
    scroll
}

fn usb_device_page(
    window: &ApplicationWindow,
    device: &CfhdbUsbDevice,
    theme_changed_action: &gio::SimpleAction,
    update_device_status_action: &gio::SimpleAction,
    device_status_indicator: &ColoredCircle,
) -> gtk::Box {
    let content_box = gtk::Box::builder()
        .hexpand(true)
        .vexpand(true)
        .orientation(Orientation::Vertical)
        .build();

    let color_badges_grid = gtk::Grid::builder()
        .hexpand(true)
        .halign(Align::Center)
        .row_homogeneous(true)
        .column_homogeneous(true)
        .valign(Align::Start)
        .orientation(Orientation::Vertical)
        .build();

    //
    let device_controls_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .valign(Align::Start)
        .halign(Align::Center)
        .margin_start(10)
        .margin_end(10)
        .margin_bottom(20)
        .margin_top(20)
        .build();
    device_controls_box.add_css_class("linked");

    //
    let color_badges_size_group0 = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);
    let color_badges_size_group1 = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

    //
    let mut color_badges_vec = vec![];
    let started_color_badge = ColorBadge::new();
    started_color_badge.set_label0(textwrap::fill("TEST_STARTED", 10));
    started_color_badge.set_group_size0(&color_badges_size_group0);
    started_color_badge.set_group_size1(&color_badges_size_group1);
    started_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&started_color_badge);

    let enabled_color_badge = ColorBadge::new();
    enabled_color_badge.set_label0(textwrap::fill("TEST_ENABLED", 10));
    enabled_color_badge.set_group_size0(&color_badges_size_group0);
    enabled_color_badge.set_group_size1(&color_badges_size_group1);
    enabled_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&enabled_color_badge);

    let driver_color_badge = ColorBadge::new();
    driver_color_badge.set_label0(textwrap::fill("TEST_DRIVER", 10));
    driver_color_badge.set_css_style("background-accent-bg");
    driver_color_badge.set_group_size0(&color_badges_size_group0);
    driver_color_badge.set_group_size1(&color_badges_size_group1);
    driver_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&driver_color_badge);

    let sysfs_busid_color_badge = ColorBadge::new();
    sysfs_busid_color_badge.set_label0(textwrap::fill("TEST_SYSFS_BUSID", 10));
    sysfs_busid_color_badge.set_css_style("background-accent-bg");
    sysfs_busid_color_badge.set_group_size0(&color_badges_size_group0);
    sysfs_busid_color_badge.set_group_size1(&color_badges_size_group1);
    sysfs_busid_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&sysfs_busid_color_badge);

    let vendor_id_color_badge = ColorBadge::new();
    vendor_id_color_badge.set_label0(textwrap::fill("TEST_VENDOR_ID", 10));
    vendor_id_color_badge.set_css_style("background-accent-bg");
    vendor_id_color_badge.set_group_size0(&color_badges_size_group0);
    vendor_id_color_badge.set_group_size1(&color_badges_size_group1);
    vendor_id_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&vendor_id_color_badge);

    let product_id_color_badge = ColorBadge::new();
    product_id_color_badge.set_label0(textwrap::fill("TEST_PRODUCT_ID", 10));
    product_id_color_badge.set_css_style("background-accent-bg");
    product_id_color_badge.set_group_size0(&color_badges_size_group0);
    product_id_color_badge.set_group_size1(&color_badges_size_group1);
    product_id_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&product_id_color_badge);
    //
    let mut last_widget: (Option<&ColorBadge>, i32) = (None, 0);
    let row_count = (color_badges_vec.len() / 2) as i32;

    for badge in color_badges_vec {
        if last_widget.0.is_none() {
            color_badges_grid.attach(badge, 0, 0, 1, 1);
        } else if last_widget.1 > row_count {
            color_badges_grid.attach_next_to(
                badge,
                Some(last_widget.0.unwrap()),
                gtk::PositionType::Top,
                1,
                1,
            )
        } else if last_widget.1 == row_count {
            color_badges_grid.attach_next_to(
                badge,
                Some(last_widget.0.unwrap()),
                gtk::PositionType::Left,
                1,
                1,
            )
        } else {
            color_badges_grid.attach_next_to(
                badge,
                Some(last_widget.0.unwrap()),
                gtk::PositionType::Bottom,
                1,
                1,
            )
        }

        last_widget.0 = Some(badge);
        last_widget.1 += 1;
    }
    //

    let control_button_start_device_button = gtk::Button::builder()
        .child(
            &gtk::Image::builder()
                .icon_name("media-playback-start-symbolic")
                .pixel_size(32)
                .build(),
        )
        .width_request(48)
        .height_request(48)
        .tooltip_text("TEST_DEVICE_CONTROL_START")
        .build();
    let control_button_stop_device_button = gtk::Button::builder()
        .child(
            &gtk::Image::builder()
                .icon_name("media-playback-stop-symbolic")
                .pixel_size(32)
                .build(),
        )
        .width_request(48)
        .height_request(48)
        .tooltip_text("TEST_DEVICE_CONTROL_STOP")
        .build();
    let control_button_enable_device_button = gtk::Button::builder()
        .child(
            &gtk::Image::builder()
                .icon_name("emblem-ok-symbolic")
                .pixel_size(32)
                .build(),
        )
        .width_request(48)
        .height_request(48)
        .tooltip_text("TEST_DEVICE_CONTROL_ENABLE")
        .build();
    let control_button_disable_device_button = gtk::Button::builder()
        .child(
            &gtk::Image::builder()
                .icon_name("edit-clear-all-symbolic")
                .pixel_size(32)
                .build(),
        )
        .width_request(48)
        .height_request(48)
        .tooltip_text("TEST_DEVICE_CONTROL_DISABLE")
        .build();

    let available_profiles_list_row = adw::PreferencesGroup::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .title("TEST_AV_PF_TITLE")
        .description("TEST_AV_PF_SUBTITLE")
        .vexpand(true)
        .hexpand(true)
        .build();

    let rows_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

    let mut profiles = device
        .available_profiles
        .0
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();
    profiles.sort_by_key(|x| x.priority);

    control_button_start_device_button.connect_clicked(clone!(#[strong] device, #[strong] window, #[strong] update_device_status_action,move |_| {
        match device.start_device() {
            Ok(_) => update_device_status_action.activate(None),
            Err(e) => {
                error_dialog(window.clone(), "TEST_DEVICE_START_ERROR", &e.to_string())
            }
        }
    }));

    control_button_enable_device_button.connect_clicked(clone!(#[strong] device, #[strong] window, #[strong] update_device_status_action,move |_| {
        match device.enable_device() {
            Ok(_) => update_device_status_action.activate(None),
            Err(e) => {
                error_dialog(window.clone(), "TEST_DEVICE_ENABLE_ERROR", &e.to_string())
            }
        }
    }));

    control_button_stop_device_button.connect_clicked(clone!(#[strong] device, #[strong] window, #[strong] update_device_status_action,move |_| {
        match device.stop_device() {
            Ok(_) => update_device_status_action.activate(None),
            Err(e) => {
                error_dialog(window.clone(), "TEST_DEVICE_STOP_ERROR", &e.to_string())
            }
        }
    }));

    control_button_disable_device_button.connect_clicked(clone!(#[strong] device, #[strong] window, #[strong] update_device_status_action,move |_| {
        match device.disable_device() {
            Ok(_) => update_device_status_action.activate(None),
            Err(e) => {
                error_dialog(window.clone(), "TEST_DEVICE_DISABLE_ERROR", &e.to_string())
            }
        }
    }));

    for profile in profiles {
        let (profiles_color_badges_size_group0, profiles_color_badges_size_group1) = (
            gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
            gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
        );
        let profile_expander_row = adw::ExpanderRow::new();
        let profile_icon = gtk::Image::builder()
            .icon_name(&profile.icon_name)
            .pixel_size(32)
            .build();
        let profile_status_icon = gtk::Image::builder()
            .icon_name("emblem-default")
            .pixel_size(24)
            .visible(false)
            .tooltip_text(t!("profile_status_icon_tooltip_text"))
            .build();
        let profile_content_row = adw::ActionRow::builder().build();
        let profile_install_button = gtk::Button::builder()
            .margin_start(5)
            .margin_top(5)
            .margin_bottom(5)
            .valign(gtk::Align::Center)
            .label(t!("profile_install_button_label"))
            .tooltip_text(t!("profile_install_button_tooltip_text"))
            .sensitive(false)
            .build();
        profile_install_button.add_css_class("suggested-action");
        let profile_remove_button = gtk::Button::builder()
            .margin_end(5)
            .margin_top(5)
            .margin_bottom(5)
            .valign(gtk::Align::Center)
            .label(t!("profile_remove_button_label"))
            .tooltip_text(t!("profile_remove_button_tooltip_text"))
            .sensitive(false)
            .build();
        let profile_action_box = gtk::Box::builder().homogeneous(true).build();
        profile_remove_button.add_css_class("destructive-action");
        profile_expander_row.add_prefix(&profile_icon);
        profile_expander_row.add_suffix(&profile_status_icon);
        profile_expander_row.set_title(&profile.i18n_desc);
        profile_expander_row.set_subtitle(&profile.codename);
        //
        let color_badge_experimental = ColorBadge::new();
        color_badge_experimental.set_label0(textwrap::fill("TEST_EXPERIMENTAL", 10));
        if profile.experimental {
            color_badge_experimental.set_label1("TEST_YES");
            color_badge_experimental.set_css_style("background-red-bg");
        } else {
            color_badge_experimental.set_label1("TEST_NO");
            color_badge_experimental.set_css_style("background-accent-bg");
        }
        color_badge_experimental.set_group_size0(&profiles_color_badges_size_group0);
        color_badge_experimental.set_group_size1(&profiles_color_badges_size_group1);
        color_badge_experimental.set_theme_changed_action(theme_changed_action);
        let color_badge_license = ColorBadge::new();
        color_badge_license.set_label0(textwrap::fill("TEST_LICENSE", 10));
        color_badge_license.set_label1(profile.license.clone());
        color_badge_license.set_css_style("background-accent-bg");
        color_badge_license.set_group_size0(&profiles_color_badges_size_group0);
        color_badge_license.set_group_size1(&profiles_color_badges_size_group1);
        color_badge_license.set_theme_changed_action(theme_changed_action);
        let badges_warp_box = gtk::Box::new(Vertical, 3);
        badges_warp_box.append(&color_badge_license);
        badges_warp_box.append(&color_badge_experimental);
        profile_content_row.add_prefix(&badges_warp_box);
        profile_action_box.append(&profile_remove_button);
        profile_action_box.append(&profile_install_button);
        profile_content_row.add_suffix(&profile_action_box);
        profile_expander_row.add_row(&profile_content_row);
        rows_size_group.add_widget(&profile_action_box);
        available_profiles_list_row.add(&profile_expander_row);
        //
        profile_install_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            profile,
            #[strong]
            update_device_status_action,
            move |_| {
                profile_modify(window.clone(), &profile, "install", &update_device_status_action);
            }
        ));
        profile_remove_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            profile,
            #[strong]
            update_device_status_action,
            move |_| {
                profile_modify(window.clone(), &profile, "remove", &update_device_status_action);
            }
        ));
        update_device_status_action.connect_activate(clone!(move |_, _| {
            let profile_status = profile.get_status();
            profile_install_button.set_sensitive(!profile_status);
            profile_remove_button.set_sensitive(profile_status);
            profile_status_icon.set_visible(profile_status);
        }));
    }

    update_device_status_action.connect_activate(clone!(
        #[strong]
        device,
        #[strong]
        device_status_indicator,
        #[strong]
        control_button_start_device_button,
        #[strong]
        control_button_stop_device_button,
        #[strong]
        control_button_enable_device_button,
        #[strong]
        control_button_disable_device_button,
        move |_, _| {
            let updated_device =
                CfhdbUsbDevice::get_device_from_busid(&device.sysfs_busid).unwrap();
            let (started, enabled) = (
                updated_device.started.unwrap_or_default(),
                updated_device.enabled,
            );
            let (color, tooltip) = match (enabled, started) {
                (true, true) => (RGBA::GREEN, "TEST_DEVICE_ACTIVE_ENABLED"),
                (false, true) => (RGBA::BLUE, "TEST_DEVICE_ACTIVE_DISABLED"),
                (true, false) => (RGBA::new(60.0, 255.0, 0.0, 1.0), "TEST_DEVICE_STOP_ENABLED"),
                (false, false) => (RGBA::RED, "TEST_DEVICE_STOP_DISABLED"),
            };
            device_status_indicator.set_color(color);
            device_status_indicator.set_tooltip_text(Some(tooltip));

            control_button_start_device_button.set_sensitive(!started);
            control_button_stop_device_button.set_sensitive(started);

            control_button_enable_device_button.set_sensitive(!enabled);
            control_button_disable_device_button.set_sensitive(enabled);

            started_color_badge.set_label1(textwrap::fill(
                if started { "TEST_YES" } else { "TEST_NO" },
                10,
            ));
            started_color_badge.set_css_style(if started {
                "background-accent-bg"
            } else {
                "background-red-bg"
            });
            enabled_color_badge.set_label1(textwrap::fill(
                if enabled { "TEST_YES" } else { "TEST_NO" },
                10,
            ));
            enabled_color_badge.set_css_style(if enabled {
                "background-accent-bg"
            } else {
                "background-red-bg"
            });
            driver_color_badge.set_label1(textwrap::fill(device.kernel_driver.as_str(), 10));
            sysfs_busid_color_badge.set_label1(textwrap::fill(&device.sysfs_busid.as_str(), 10));
            vendor_id_color_badge.set_label1(textwrap::fill(&device.vendor_id.as_str(), 10));
            product_id_color_badge.set_label1(textwrap::fill(&device.product_id.as_str(), 10));
        }
    ));

    update_device_status_action.activate(None);

    device_controls_box.append(&control_button_start_device_button);

    device_controls_box.append(&control_button_stop_device_button);

    device_controls_box.append(&control_button_enable_device_button);

    device_controls_box.append(&control_button_disable_device_button);

    content_box.append(&color_badges_grid);
    content_box.append(&device_controls_box);
    content_box.append(&available_profiles_list_row);

    content_box
}

fn profile_modify(window: ApplicationWindow, profile: &CfhdbUsbProfile, opreation: &str, update_device_status_action: &gio::SimpleAction) {
    let (log_loop_sender, log_loop_receiver) = async_channel::unbounded();
    let log_loop_sender: async_channel::Sender<ChannelMsg> = log_loop_sender.clone();

    let profile_modify_log_terminal_buffer = gtk::TextBuffer::builder().build();

    let profile_modify_log_terminal = gtk::TextView::builder()
        .vexpand(true)
        .hexpand(true)
        .editable(false)
        .buffer(&profile_modify_log_terminal_buffer)
        .build();

    let profile_modify_log_terminal_scroll = gtk::ScrolledWindow::builder()
        .width_request(400)
        .height_request(200)
        .vexpand(true)
        .hexpand(true)
        .child(&profile_modify_log_terminal)
        .build();

    let profile_modify_dialog = adw::AlertDialog::builder()
        .extra_child(&profile_modify_log_terminal_scroll)
        .width_request(400)
        .height_request(200)
        .heading(t!(format!("profile_{}_dialog_heading", opreation)))
        .can_close(false)
        .build();
    profile_modify_dialog.add_response(
        "profile_modify_dialog_ok",
        &t!(format!("profile_{}_dialog_ok_label", opreation)).to_string(),
    );
    profile_modify_dialog.add_response(
        "profile_modify_dialog_reboot",
        &t!(format!("profile_{}_dialog_reboot_label", opreation)).to_string(),
    );
    profile_modify_dialog.set_response_appearance(
        "profile_modify_dialog_reboot",
        adw::ResponseAppearance::Suggested,
    );

    //

    let string_opreation = String::from(opreation);

    thread::spawn(clone!(
        #[strong]
        profile,
        #[strong]
        string_opreation,
        move || {
            let script = match string_opreation.as_str() {
                "install" => profile.install_script,
                "remove" => profile.remove_script,
                _ => panic!(),
            };
            match script {
                Some(t) => match profile.packages {
                    Some(a) => {
                        let package_list = a.join(" ");
                        let modify_command =
                            format!("apt-get --assume-no {} {}", &string_opreation, package_list);
                        run_in_lock_script(
                            &log_loop_sender,
                            &format!("#! /bin/bash\nset -e\n{}\n{}", modify_command, t),
                        );
                    }
                    None => {
                        run_in_lock_script(
                            &log_loop_sender,
                            &format!("#! /bin/bash\nset -e\n{}", t),
                        );
                    }
                },
                None => match profile.packages {
                    Some(a) => {
                        let package_list = a.join(" ");
                        let modify_command =
                            format!("apt-get --assume-no {} {}", &string_opreation, package_list);
                        run_in_lock_script(
                            &log_loop_sender,
                            &format!("#! /bin/bash\nset -e\n{}", modify_command),
                        );
                    }
                    None => {
                        log_loop_sender
                            .send_blocking(ChannelMsg::SuccessMsg)
                            .unwrap();
                    }
                },
            }
        }
    ));

    let log_loop_context = MainContext::default();
    // The main loop executes the asynchronous block
    log_loop_context.spawn_local(clone!(
        #[strong]
        profile_modify_log_terminal_buffer,
        #[strong]
        profile_modify_dialog,
        #[strong]
        string_opreation,
        async move {
            while let Ok(state) = log_loop_receiver.recv().await {
                match state {
                    ChannelMsg::OutputLine(line) => profile_modify_log_terminal_buffer.insert(
                        &mut profile_modify_log_terminal_buffer.end_iter(),
                        &("\n".to_string() + &line),
                    ),
                    ChannelMsg::SuccessMsg => {
                        if get_current_username().unwrap() == "pikaos" {
                            profile_modify_dialog
                                .set_response_enabled("profile_modify_dialog_reboot", false);
                        } else {
                            profile_modify_dialog
                                .set_response_enabled("profile_modify_dialog_reboot", true);
                        }
                        profile_modify_dialog
                            .set_response_enabled("profile_modify_dialog_reboot", true);
                        profile_modify_dialog.set_body(
                            &t!(format!(
                                "profile_{}_dialog_body_successful",
                                &string_opreation
                            ))
                            .to_string(),
                        );
                    }
                    ChannelMsg::FailMsg => {
                        profile_modify_dialog
                            .set_response_enabled("profile_modify_dialog_ok", true);
                        profile_modify_dialog.set_body(&t!(format!(
                            "profile_{}_dialog_body_failed",
                            &string_opreation
                        )
                        .to_string()));
                        profile_modify_dialog
                            .set_response_enabled("profile_modify_dialog_reboot", false);
                    }
                    ChannelMsg::SuccessMsgDeviceFetch(_, _) => {
                        panic!();
                    }
                }
            }
        }
    ));

    profile_modify_dialog.set_response_enabled("profile_modify_dialog_ok", false);
    profile_modify_dialog.set_response_enabled("profile_modify_dialog_reboot", false);
    let dialog_closure = clone!(
        #[strong]
        profile_modify_dialog,
        #[strong]
        update_device_status_action,
        move |choice: glib::GString| {
            match choice.as_str() {
                "profile_modify_dialog_reboot" => {
                    Command::new("systemctl")
                        .arg("reboot")
                        .spawn()
                        .expect("systemctl reboot failed to start");
                }
                _ => {
                    profile_modify_dialog.force_close();
                    update_device_status_action.activate(None);
                }
            }
        }
    );
    profile_modify_dialog.choose(&window, gio::Cancellable::NONE, dialog_closure);
}