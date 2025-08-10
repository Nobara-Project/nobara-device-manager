use crate::{
    build_ui::{color_badge::ColorBadge, get_current_font, wrap_text},
    cfhdb::dmi::{PreCheckedDmiInfo, PreCheckedDmiProfile},
};
use adw::{prelude::*, *};
use gtk::{
    glib::{clone, MainContext},
    Align, Orientation,
    Orientation::Vertical,
    ScrolledWindow,
};
use pikd_pharser_rs::{init_pikd_with_duct, PikChannel};

use std::{process::Command, rc::Rc, sync::Arc, thread};

pub fn create_dmi_class(
    window: &ApplicationWindow,
    info: &PreCheckedDmiInfo,
    theme_changed_action: &gio::SimpleAction,
    update_info_status_action: &gio::SimpleAction,
) -> (String, ScrolledWindow) {
    // Update all profiles' installation status before creating the UI
    for profile in &info.profiles {
        profile.update_installed();
    }

    let navigation_view = dmi_info_page(
            &window,
            &info,
            &theme_changed_action,
            &update_info_status_action,
    );

    let scroll = gtk::ScrolledWindow::builder()
        .max_content_width(650)
        .min_content_width(300)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&navigation_view)
        .build();
    //
    let info_content = &info.info;
    let info_title = info_content.product_name.clone();

    (info_title, scroll)
}

fn dmi_info_page(
    window: &ApplicationWindow,
    info: &PreCheckedDmiInfo,
    theme_changed_action: &gio::SimpleAction,
    update_info_status_action: &gio::SimpleAction,
) -> gtk::Box {
    let info_content = &info.info;
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
    let color_badges_size_group0 = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);
    let color_badges_size_group1 = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

    let mut color_badges_vec = vec![];

    //
    let bios_date_color_badge = ColorBadge::new();
    bios_date_color_badge.set_label0(textwrap::fill(&t!("info_bios_date"), 10));
    bios_date_color_badge.set_label1(textwrap::fill(&info_content.bios_date, 10));
    bios_date_color_badge.set_css_style("background-accent-bg");
    bios_date_color_badge.set_group_size0(&color_badges_size_group0);
    bios_date_color_badge.set_group_size1(&color_badges_size_group1);
    bios_date_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&bios_date_color_badge);
    //

    let bios_release_color_badge = ColorBadge::new();
    bios_release_color_badge.set_label0(textwrap::fill(&t!("info_bios_release"), 10));
    bios_release_color_badge.set_label1(textwrap::fill(&info_content.bios_release, 10));
    bios_release_color_badge.set_css_style("background-accent-bg");
    bios_release_color_badge.set_group_size0(&color_badges_size_group0);
    bios_release_color_badge.set_group_size1(&color_badges_size_group1);
    bios_release_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&bios_release_color_badge);
    //

    let bios_vendor_color_badge = ColorBadge::new();
    bios_vendor_color_badge.set_label0(textwrap::fill(&t!("info_bios_vendor"), 10));
    bios_vendor_color_badge.set_label1(textwrap::fill(&info_content.bios_vendor, 10));
    bios_vendor_color_badge.set_css_style("background-accent-bg");
    bios_vendor_color_badge.set_group_size0(&color_badges_size_group0);
    bios_vendor_color_badge.set_group_size1(&color_badges_size_group1);
    bios_vendor_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&bios_vendor_color_badge);
    //

    let bios_version_color_badge = ColorBadge::new();
    bios_version_color_badge.set_label0(textwrap::fill(&t!("info_bios_version"), 10));
    bios_version_color_badge.set_label1(textwrap::fill(&info_content.bios_version, 10));
    bios_version_color_badge.set_css_style("background-accent-bg");
    bios_version_color_badge.set_group_size0(&color_badges_size_group0);
    bios_version_color_badge.set_group_size1(&color_badges_size_group1);
    bios_version_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&bios_version_color_badge);
    //

    let board_asset_tag_color_badge = ColorBadge::new();
    board_asset_tag_color_badge.set_label0(textwrap::fill(&t!("info_board_asset_tag"), 10));
    board_asset_tag_color_badge.set_label1(textwrap::fill(&info_content.board_asset_tag, 10));
    board_asset_tag_color_badge.set_css_style("background-accent-bg");
    board_asset_tag_color_badge.set_group_size0(&color_badges_size_group0);
    board_asset_tag_color_badge.set_group_size1(&color_badges_size_group1);
    board_asset_tag_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&board_asset_tag_color_badge);
    //

    let board_name_color_badge = ColorBadge::new();
    board_name_color_badge.set_label0(textwrap::fill(&t!("info_board_name"), 10));
    board_name_color_badge.set_label1(textwrap::fill(&info_content.board_name, 10));
    board_name_color_badge.set_css_style("background-accent-bg");
    board_name_color_badge.set_group_size0(&color_badges_size_group0);
    board_name_color_badge.set_group_size1(&color_badges_size_group1);
    board_name_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&board_name_color_badge);
    //

    let board_vendor_color_badge = ColorBadge::new();
    board_vendor_color_badge.set_label0(textwrap::fill(&t!("info_board_vendor"), 10));
    board_vendor_color_badge.set_label1(textwrap::fill(&info_content.board_vendor, 10));
    board_vendor_color_badge.set_css_style("background-accent-bg");
    board_vendor_color_badge.set_group_size0(&color_badges_size_group0);
    board_vendor_color_badge.set_group_size1(&color_badges_size_group1);
    board_vendor_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&board_vendor_color_badge);

    //

    let board_version_color_badge = ColorBadge::new();
    board_version_color_badge.set_label0(textwrap::fill(&t!("info_board_version"), 10));
    board_version_color_badge.set_label1(textwrap::fill(&info_content.bios_version, 10));
    board_version_color_badge.set_css_style("background-accent-bg");
    board_version_color_badge.set_group_size0(&color_badges_size_group0);
    board_version_color_badge.set_group_size1(&color_badges_size_group1);
    board_version_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&board_version_color_badge);
    //

    let product_family_color_badge = ColorBadge::new();
    product_family_color_badge.set_label0(textwrap::fill(&t!("info_product_family"), 10));
    product_family_color_badge.set_label1(textwrap::fill(&info_content.product_family, 10));
    product_family_color_badge.set_css_style("background-accent-bg");
    product_family_color_badge.set_group_size0(&color_badges_size_group0);
    product_family_color_badge.set_group_size1(&color_badges_size_group1);
    product_family_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&product_family_color_badge);
    //

    let product_name_color_badge = ColorBadge::new();
    product_name_color_badge.set_label0(textwrap::fill(&t!("info_product_name"), 10));
    product_name_color_badge.set_label1(textwrap::fill(&info_content.product_name, 10));
    product_name_color_badge.set_css_style("background-accent-bg");
    product_name_color_badge.set_group_size0(&color_badges_size_group0);
    product_name_color_badge.set_group_size1(&color_badges_size_group1);
    product_name_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&product_name_color_badge);
    //

    let product_sku_color_badge = ColorBadge::new();
    product_sku_color_badge.set_label0(textwrap::fill(&t!("info_product_sku"), 10));
    product_sku_color_badge.set_label1(textwrap::fill(&info_content.product_sku, 10));
    product_sku_color_badge.set_css_style("background-accent-bg");
    product_sku_color_badge.set_group_size0(&color_badges_size_group0);
    product_sku_color_badge.set_group_size1(&color_badges_size_group1);
    product_sku_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&product_sku_color_badge);
    //

    let product_version_color_badge = ColorBadge::new();
    product_version_color_badge.set_label0(textwrap::fill(&t!("info_product_version"), 10));
    product_version_color_badge.set_label1(textwrap::fill(&info_content.product_version, 10));
    product_version_color_badge.set_css_style("background-accent-bg");
    product_version_color_badge.set_group_size0(&color_badges_size_group0);
    product_version_color_badge.set_group_size1(&color_badges_size_group1);
    product_version_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&product_version_color_badge);
    //

    let sys_vendor_color_badge = ColorBadge::new();
    sys_vendor_color_badge.set_label0(textwrap::fill(&t!("info_sys_vendor"), 10));
    sys_vendor_color_badge.set_label1(textwrap::fill(&info_content.sys_vendor, 10));
    sys_vendor_color_badge.set_css_style("background-accent-bg");
    sys_vendor_color_badge.set_group_size0(&color_badges_size_group0);
    sys_vendor_color_badge.set_group_size1(&color_badges_size_group1);
    sys_vendor_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&sys_vendor_color_badge);
    //
    let mut last_widget: (Option<&ColorBadge>, i32) = (None, 0);
    let mut next_position = gtk::PositionType::Bottom;
    let max_row_count = 4;

    for badge in color_badges_vec {
        if last_widget.0.is_none() {
            color_badges_grid.attach(badge, 0, 0, 1, 1);
        } else if last_widget.1 < max_row_count {
            color_badges_grid.attach_next_to(
                badge,
                Some(last_widget.0.unwrap()),
                next_position,
                1,
                1,
            );
            last_widget.1 += 1;
        } else if last_widget.1 == max_row_count {
            color_badges_grid.attach_next_to(
                badge,
                Some(last_widget.0.unwrap()),
                gtk::PositionType::Right,
                1,
                1,
            );
            last_widget.1 = 0;
            if next_position == gtk::PositionType::Top {
                next_position = gtk::PositionType::Bottom
            } else {
                next_position = gtk::PositionType::Top
            }
        }

        last_widget.0 = Some(badge);
    }
    //

    let available_profiles_list_row = adw::PreferencesGroup::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .valign(gtk::Align::End)
        .title(t!("available_profiles_title"))
        .description(t!("available_profiles_subtitle"))
        .vexpand(true)
        .hexpand(true)
        .build();

    let rows_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

    let mut profiles = info.profiles.clone();
    profiles.sort_by_key(|x| x.profile().priority);

    let mut normal_profiles = vec![];
    let mut veiled_profiles = vec![];

    for profile in profiles.clone() {
        let profile_content = profile.profile();
        let (profiles_color_badges_size_group0, profiles_color_badges_size_group1) = (
            gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
            gtk::SizeGroup::new(gtk::SizeGroupMode::Both),
        );
        let profile_expander_row = adw::ExpanderRow::new();
        let profile_icon = gtk::Image::builder()
            .icon_name(&profile_content.icon_name)
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
        profile_expander_row.set_title(&profile_content.i18n_desc);
        profile_expander_row.set_subtitle(&profile_content.codename);
        //
        let color_badge_experimental = ColorBadge::new();
        color_badge_experimental.set_label0(textwrap::fill(&t!("profile_experimental"), 10));
        if profile_content.experimental {
            color_badge_experimental.set_label1(t!("status_yes"));
            color_badge_experimental.set_css_style("background-red-bg");
        } else {
            color_badge_experimental.set_label1(t!("status_no"));
            color_badge_experimental.set_css_style("background-accent-bg");
        }
        color_badge_experimental.set_group_size0(&profiles_color_badges_size_group0);
        color_badge_experimental.set_group_size1(&profiles_color_badges_size_group1);
        color_badge_experimental.set_theme_changed_action(theme_changed_action);
        let color_badge_license = ColorBadge::new();
        color_badge_license.set_label0(textwrap::fill(&t!("profile_license"), 10));
        color_badge_license.set_label1(profile_content.license.clone());
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
        //
        let profiles_rc = Rc::new(profiles.clone());
        profile_install_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_info_status_action,
            #[strong]
            profile,
            #[strong]
            profiles_rc,
            #[strong]
            theme_changed_action,
            move |_| {
                profile_modify(
                    window.clone(),
                    &update_info_status_action,
                    &profile,
                    &profiles_rc,
                    "install",
                    &theme_changed_action,
                );
            }
        ));
        profile_remove_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_info_status_action,
            #[strong]
            profile,
            #[strong]
            profiles_rc,
            #[strong]
            theme_changed_action,
            move |_| {
                profile_modify(
                    window.clone(),
                    &update_info_status_action,
                    &profile,
                    &profiles_rc,
                    "remove",
                    &theme_changed_action,
                );
            }
        ));
        //
        if profile_content.veiled {
            veiled_profiles.push(profile_expander_row);
        } else {
            normal_profiles.push(profile_expander_row);
        }
        //
        update_info_status_action.connect_activate(clone!(move |_, _| {
            let profile_status = profile.installed();
            profile_install_button.set_sensitive(!profile_status);
            if profile_content.removable {
                profile_remove_button.set_sensitive(profile_status);
            } else {
                profile_remove_button.set_sensitive(false);
            }
            profile_status_icon.set_visible(profile_status);
        }));
    }

    update_info_status_action.activate(None);

    content_box.append(&color_badges_grid);
    for widget in normal_profiles {
        available_profiles_list_row.add(&widget);
    }
    content_box.append(&available_profiles_list_row);
    if !veiled_profiles.is_empty() {
        let veiled_profiles_list_row = gtk::ListBox::builder()
            .vexpand(true)
            .hexpand(true)
            .margin_top(20)
            .margin_end(20)
            .build();
        veiled_profiles_list_row.add_css_class("boxed-list");
        let label = gtk::Label::new(Some(&t!("viel_expander_label")));
        label.add_css_class("title-1");
        let veil_expander = gtk::Expander::builder()
            .child(&veiled_profiles_list_row)
            .valign(gtk::Align::Start)
            .vexpand(true)
            .label_widget(&label)
            .margin_top(20)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .build();
        for widget in veiled_profiles {
            veiled_profiles_list_row.append(&widget);
        }
        content_box.append(&veil_expander);
    }

    content_box
}

pub fn profile_modify(
    window: ApplicationWindow,
    update_info_status_action: &gio::SimpleAction,
    profile: &Arc<PreCheckedDmiProfile>,
    all_profiles: &Rc<Vec<Arc<PreCheckedDmiProfile>>>,
    opreation: &str,
    theme_changed_action: &gio::SimpleAction,
) {
    let profile_content = profile.profile();

    let string_opreation = String::from(opreation);

    //

    let (process_log_sender, process_log_receiver) = async_channel::unbounded();

    let log_file_path = format!(
        "/tmp/cfhdb-opreation_{}.log",
        chrono::offset::Local::now().format("%Y-%m-%d_%H:%M")
    );

    let pikd_dialog_child_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let pikd_dialog_progress_bar = circularprogressbar_rs::CircularProgressBar::new();
    pikd_dialog_progress_bar.set_line_width(10.0);
    pikd_dialog_progress_bar.set_fill_radius(true);
    pikd_dialog_progress_bar.set_hexpand(true);
    pikd_dialog_progress_bar.set_vexpand(true);
    pikd_dialog_progress_bar.set_width_request(200);
    pikd_dialog_progress_bar.set_height_request(200);
    #[allow(deprecated)]
    pikd_dialog_progress_bar.set_progress_fill_color(
        window
            .style_context()
            .lookup_color("accent_bg_color")
            .unwrap(),
    );
    #[allow(deprecated)]
    pikd_dialog_progress_bar.set_radius_fill_color(
        window
            .style_context()
            .lookup_color("headerbar_bg_color")
            .unwrap(),
    );
    #[warn(deprecated)]
    pikd_dialog_progress_bar.set_progress_font(get_current_font());
    pikd_dialog_progress_bar.set_center_text(t!("progress_bar_circle_center_text"));
    pikd_dialog_progress_bar.set_fraction_font_size(24);
    pikd_dialog_progress_bar.set_center_text_font_size(8);
    theme_changed_action.connect_activate(clone!(
        #[strong]
        window,
        #[strong]
        pikd_dialog_progress_bar,
        move |_, _| {
            #[allow(deprecated)]
            pikd_dialog_progress_bar.set_progress_fill_color(
                window
                    .style_context()
                    .lookup_color("accent_bg_color")
                    .unwrap(),
            );
            #[allow(deprecated)]
            pikd_dialog_progress_bar.set_radius_fill_color(
                window
                    .style_context()
                    .lookup_color("headerbar_bg_color")
                    .unwrap(),
            );
            #[warn(deprecated)]
            pikd_dialog_progress_bar.set_progress_font(get_current_font());
        }
    ));

    let apt_speed_label = gtk::Label::builder()
        .halign(Align::Center)
        .margin_top(10)
        .margin_bottom(10)
        .width_request(150)
        .height_request(150)
        .build();

    pikd_dialog_child_box.append(&pikd_dialog_progress_bar);
    pikd_dialog_child_box.append(&apt_speed_label);

    let pikd_dialog = adw::AlertDialog::builder()
        .extra_child(&pikd_dialog_child_box)
        .heading(match string_opreation.as_str() {
            "install" => t!("profile_install_dialog_heading"),
            _ => t!("profile_remove_dialog_heading"),
        })
        .width_request(600)
        .height_request(600)
        .build();

    pikd_dialog.add_response(
        "pikd_dialog_ok",
        &t!("profile_install_dialog_ok_label").to_string(),
    );
    pikd_dialog.add_response(
        "pikd_dialog_reboot",
        &t!("profile_install_dialog_reboot_label").to_string(),
    );

    let pikd_dialog_child_box_done = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let pikd_log_image = gtk::Image::builder()
        .pixel_size(128)
        .halign(Align::Center)
        .build();

    let pikd_log_button = gtk::Button::builder()
        .label(t!("pikd_dialog_open_log_file"))
        .halign(Align::Center)
        .margin_start(15)
        .margin_end(15)
        .margin_top(15)
        .margin_bottom(15)
        .build();

    pikd_dialog_child_box_done.append(&pikd_log_image);
    pikd_dialog_child_box_done.append(&pikd_log_button);

    pikd_dialog.set_response_enabled("pikd_dialog_ok", false);
    pikd_dialog.set_close_response("pikd_dialog_ok");

    pikd_dialog.set_response_enabled("pikd_dialog_reboot", false);

    let process_log_context = MainContext::default();
    // The main loop executes the asynchronous block
    process_log_context.spawn_local(clone!(
        #[weak]
        pikd_dialog_progress_bar,
        #[weak]
        apt_speed_label,
        #[weak]
        pikd_dialog,
        #[weak]
        pikd_dialog_child_box,
        #[strong]
        pikd_dialog_child_box_done,
        #[strong]
        pikd_log_image,
        #[strong]
        update_info_status_action,
        #[strong]
        string_opreation,
        async move {
            while let Ok(state) = process_log_receiver.recv().await {
                match state {
                    PikChannel::Status(status) => {
                        if status {
                            pikd_dialog_child_box.set_visible(false);
                            pikd_log_image.set_icon_name(Some("face-cool-symbolic"));
                            pikd_dialog.set_extra_child(Some(&pikd_dialog_child_box_done));
                            pikd_dialog.set_title(&match string_opreation.as_str() {
                                "install" => {
                                    t!("profile_install_dialog_body_successful").to_string()
                                }
                                _ => t!("profile_install_dialog_body_failed").to_string(),
                            });
                            pikd_dialog.set_response_enabled("pikd_dialog_ok", true);
                        } else {
                            pikd_dialog_child_box.set_visible(false);
                            pikd_log_image.set_icon_name(Some("dialog-error-symbolic"));
                            pikd_dialog.set_extra_child(Some(&pikd_dialog_child_box_done));
                            pikd_dialog.set_title(&match string_opreation.as_str() {
                                "install" => {
                                    t!("profile_remove_dialog_body_successful").to_string()
                                }
                                _ => t!("profile_remove_dialog_body_failed").to_string(),
                            });
                            pikd_dialog.set_response_enabled("pikd_dialog_ok", true);
                            pikd_dialog.set_response_enabled("pikd_dialog_open_log_file", true);
                        }
                        update_info_status_action.activate(None);
                    }
                    PikChannel::DownloadStats(stats) => {
                        pikd_dialog_progress_bar.set_fraction(stats.total_percent / 100.0);
                        let total_downloaded =
                            format!("{} {}", stats.total_downloaded.0, stats.total_downloaded.1);
                        let total_size =
                            format!("{} {}", stats.total_size.0.to_string(), stats.total_size.1);
                        let total_speed = format!(
                            "{} {}",
                            stats.total_speed.0.to_string(),
                            stats.total_speed.1
                        );
                        pikd_dialog.set_body(&wrap_text(
                            &format!(
                                "{}: {}% ({} out of {}, {})",
                                t!("pikd_dialog_total"),
                                stats.total_percent,
                                total_downloaded,
                                total_size,
                                total_speed
                            ),
                            30,
                        ));
                        let package_downloaded = format!(
                            "{} {}",
                            stats.package_downloaded.0.to_string(),
                            stats.package_downloaded.1
                        );
                        let package_size = format!(
                            "{} {}",
                            stats.package_size.0.to_string(),
                            stats.package_size.1
                        );
                        let package_speed = format!(
                            "{} {}",
                            stats.package_speed.0.to_string(),
                            stats.package_speed.1
                        );
                        apt_speed_label.set_label(&wrap_text(
                            &format!(
                                "{}: {}% ({} out of {}, {})",
                                stats.package_name,
                                stats.package_percent,
                                package_downloaded,
                                package_size,
                                package_speed
                            ),
                            30,
                        ));
                    }
                    PikChannel::InfoMessage(msg) => apt_speed_label.set_label(&wrap_text(&msg, 30)),
                }
            }
        }
    ));

    pikd_log_button.connect_clicked(clone!(
        #[strong]
        log_file_path,
        move |_| {
            let _ = Command::new("xdg-open")
                .arg(log_file_path.to_owned())
                .spawn();
        }
    ));

    thread::spawn(clone!(
        #[strong]
        profile_content,
        #[strong]
        process_log_sender,
        #[strong]
        log_file_path,
        move || {
            //
            let process_log_sender_clone0 = process_log_sender.clone();
            let process_log_sender_clone1 = process_log_sender.clone();
            let log_file_path_clone0 = log_file_path.clone();
            match string_opreation.as_str() {
                "install" => {
                    let script = profile_content.install_script;
                    match script {
                        Some(t) => match profile_content.packages {
                            Some(a) => {
                                let package_list = a.join(" ");
                                let command = init_pikd_with_duct(
                                    duct::cmd!(
                                        "pkexec",
                                        "bash",
                                        "-c",
                                        &format!(
                                            r###"
#! /bin/bash
set -e
                                
echo "DOWNLOAD {PLIST}" | nc -U /var/run/pik.sock
echo "INSTALL {PLIST}" | nc -U /var/run/pik.sock
                                
{PRESC}
                                "###,
                                            PLIST = package_list,
                                            PRESC = t
                                        )
                                    ),
                                    process_log_sender_clone0,
                                    &log_file_path_clone0,
                                );
                                match command {
                                    Ok(_) => {}
                                    Err(_) => {
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::InfoMessage(
                                                t!("pikd_dialog_status_error_perms").to_string(),
                                            ))
                                            .unwrap();
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::Status(false))
                                            .unwrap()
                                    }
                                }
                            }
                            None => {
                                let command = init_pikd_with_duct(
                                    duct::cmd!(
                                        "pkexec",
                                        "bash",
                                        "-c",
                                        &format!(
                                            r###"
#! /bin/bash
set -e
                                
{PRESC}
                                "###,
                                            PRESC = t
                                        )
                                    ),
                                    process_log_sender_clone0,
                                    &log_file_path_clone0,
                                );
                                match command {
                                    Ok(_) => {}
                                    Err(_) => {
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::InfoMessage(
                                                t!("pikd_dialog_status_error_perms").to_string(),
                                            ))
                                            .unwrap();
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::Status(false))
                                            .unwrap()
                                    }
                                }
                            }
                        },
                        None => match profile_content.packages {
                            Some(a) => {
                                let package_list = a.join(" ");
                                let command = init_pikd_with_duct(
                                    duct::cmd!(
                                        "pkexec",
                                        "bash",
                                        "-c",
                                        &format!(
                                            r###"
#! /bin/bash
set -e
                                
echo "DOWNLOAD {PLIST}" | nc -U /var/run/pik.sock
echo "INSTALL {PLIST}" | nc -U /var/run/pik.sock
                                "###,
                                            PLIST = package_list
                                        )
                                    ),
                                    process_log_sender_clone0,
                                    &log_file_path_clone0,
                                );
                                match command {
                                    Ok(_) => {}
                                    Err(_) => {
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::InfoMessage(
                                                t!("pikd_dialog_status_error_perms").to_string(),
                                            ))
                                            .unwrap();
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::Status(false))
                                            .unwrap()
                                    }
                                }
                            }
                            None => process_log_sender_clone1
                                .send_blocking(PikChannel::Status(true))
                                .unwrap(),
                        },
                    }
                }
                "remove" => {
                    let script = profile_content.remove_script;
                    match script {
                        Some(t) => match profile_content.packages {
                            Some(a) => {
                                let package_list = a.join(" ");
                                let command = init_pikd_with_duct(
                                    duct::cmd!(
                                        "pkexec",
                                        "bash",
                                        "-c",
                                        &format!(
                                            r###"
#! /bin/bash
set -e
                                
echo "REMOVE --purge {PLIST}" | nc -U /var/run/pik.sock
                                
{PRESC}
                                "###,
                                            PLIST = package_list,
                                            PRESC = t
                                        )
                                    ),
                                    process_log_sender_clone0,
                                    &log_file_path_clone0,
                                );
                                match command {
                                    Ok(_) => {}
                                    Err(_) => {
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::InfoMessage(
                                                t!("pikd_dialog_status_error_perms").to_string(),
                                            ))
                                            .unwrap();
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::Status(false))
                                            .unwrap()
                                    }
                                }
                            }
                            None => {
                                let command = init_pikd_with_duct(
                                    duct::cmd!(
                                        "pkexec",
                                        "bash",
                                        "-c",
                                        &format!(
                                            r###"
#! /bin/bash
set -e
                                
{PRESC}
                                "###,
                                            PRESC = t
                                        )
                                    ),
                                    process_log_sender_clone0,
                                    &log_file_path_clone0,
                                );
                                match command {
                                    Ok(_) => {}
                                    Err(_) => {
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::InfoMessage(
                                                t!("pikd_dialog_status_error_perms").to_string(),
                                            ))
                                            .unwrap();
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::Status(false))
                                            .unwrap()
                                    }
                                }
                            }
                        },
                        None => match profile_content.packages {
                            Some(a) => {
                                let package_list = a.join(" ");
                                let command = init_pikd_with_duct(
                                    duct::cmd!(
                                        "pkexec",
                                        "bash",
                                        "-c",
                                        &format!(
                                            r###"
#! /bin/bash
set -e
                                
echo "REMOVE --purge {PLIST}" | nc -U /var/run/pik.sock
                                "###,
                                            PLIST = package_list
                                        )
                                    ),
                                    process_log_sender_clone0,
                                    &log_file_path_clone0,
                                );
                                match command {
                                    Ok(_) => {}
                                    Err(_) => {
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::InfoMessage(
                                                t!("pikd_dialog_status_error_perms").to_string(),
                                            ))
                                            .unwrap();
                                        process_log_sender_clone1
                                            .send_blocking(PikChannel::Status(false))
                                            .unwrap()
                                    }
                                }
                            }
                            None => process_log_sender_clone1
                                .send_blocking(PikChannel::Status(true))
                                .unwrap(),
                        },
                    }
                }
                _ => panic!(),
            };
        }
    ));

    let dialog_closure = clone!(
        #[strong]
        pikd_dialog,
        #[strong]
        all_profiles,
        #[strong]
        update_info_status_action,
        move |choice: glib::GString| {
            match choice.as_str() {
                "pikd_dialog_reboot" => {
                    Command::new("systemctl")
                        .arg("reboot")
                        .spawn()
                        .expect("systemctl reboot failed to start");
                }
                _ => {
                    pikd_dialog.force_close();
                    for a_profile in all_profiles.iter() {
                        a_profile.update_installed();
                    }
                    update_info_status_action.activate(None);
                }
            }
        }
    );
    pikd_dialog.choose(&window, gio::Cancellable::NONE, dialog_closure);
}
