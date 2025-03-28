use crate::cfhdb::pci::{get_pci_devices, get_pci_profiles_from_url};
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
use libcfhdb::pci::CfhdbPciDevice;
use libcfhdb::usb::CfhdbUsbDevice;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use super::color_badge::ColorBadge;
use super::colored_circle::{self, ColoredCircle};

pub fn main_content(
    window: &adw::ApplicationWindow,
    hashmap_pci: HashMap<String, Vec<CfhdbPciDevice>>,
    hashmap_usb: HashMap<String, Vec<CfhdbUsbDevice>>,
) -> adw::OverlaySplitView {
    let theme_changed_action = gio::SimpleAction::new("theme_changed", None);
    theme_changed_thread(&theme_changed_action);
    let window_breakpoint = adw::Breakpoint::new(BreakpointCondition::new_length(
        BreakpointConditionLengthType::MaxWidth,
        900.0,
        LengthUnit::Sp,
    ));

    let main_content_overlay_split_view = adw::OverlaySplitView::builder()
        .sidebar_width_unit(adw::LengthUnit::Sp)
        .max_sidebar_width(225.0)
        .min_sidebar_width(115.0)
        .build();

    let window_banner = Banner::builder().revealed(false).build();

    let window_stack = gtk::Stack::builder()
        .transition_type(StackTransitionType::SlideUpDown)
        .build();

    main_content_overlay_split_view.set_content(Some(&main_content_content(
        &window,
        &window_banner,
        &window_stack,
        &main_content_overlay_split_view,
        &window_breakpoint,
    )));

    let mut is_first = true;
    let mut pci_buttons = vec![];
    let mut usb_buttons = vec![];
    let null_toggle_sidebar = ToggleButton::default();

    let mut hashmap_pci: Vec<(&String, &Vec<CfhdbPciDevice>)> = hashmap_pci.iter().collect();
    hashmap_pci.sort_by(|a, b| {
        let a_class = t!(format!("pci_class_name_{}", a.0))
            .to_string()
            .to_lowercase();
        let b_class = t!(format!("pci_class_name_{}", b.0))
            .to_string()
            .to_lowercase();
        a_class.cmp(&b_class)
    });

    let mut hashmap_usb: Vec<(&String, &Vec<CfhdbUsbDevice>)> = hashmap_usb.iter().collect();
    hashmap_usb.sort_by(|a, b| {
        let a_class = t!(format!("usb_class_name_{}", a.0))
            .to_string()
            .to_lowercase();
        let b_class = t!(format!("usb_class_name_{}", b.0))
            .to_string()
            .to_lowercase();
        a_class.cmp(&b_class)
    });

    for (class, devices) in hashmap_pci {
        let class = format!("pci_class_name_{}", class);
        let class_i18n = t!(class).to_string();
        window_stack.add_titled(
            &create_pci_class(&devices, &class_i18n, &theme_changed_action),
            Some(&class),
            &class_i18n,
        );
        pci_buttons.push(custom_stack_selection_button(
            &window_stack,
            if is_first {
                is_first = false;
                true
            } else {
                false
            },
            class.clone(),
            class_i18n,
            "".into(),
            &null_toggle_sidebar,
        ));
    }

    for (class, devices) in hashmap_usb {
        let class = format!("usb_class_name_{}", class);
        window_stack.add_titled(
            &gtk::Label::new(Some(&class)),
            Some(&class),
            &t!(class).to_string(),
        );
        usb_buttons.push(custom_stack_selection_button(
            &window_stack,
            false,
            class.clone(),
            t!(class).to_string(),
            "".into(),
            &null_toggle_sidebar,
        ));
    }

    main_content_overlay_split_view.set_sidebar(Some(&main_content_sidebar(
        &window_stack,
        &pci_buttons,
        &usb_buttons,
    )));

    window_breakpoint.add_setter(
        &main_content_overlay_split_view,
        "collapsed",
        Some(&true.to_value()),
    );

    window.add_breakpoint(window_breakpoint);

    let internet_check_closure = clone!(
        #[strong]
        window_banner,
        move |state: bool| {
            window_banner.set_title(&t!("banner_text_no_internet"));
            window_banner.set_revealed(!state);
        }
    );

    internet_check_loop(internet_check_closure);

    main_content_overlay_split_view
}

fn main_content_sidebar(
    stack: &gtk::Stack,
    pci_buttons: &Vec<ToggleButton>,
    usb_buttons: &Vec<ToggleButton>,
) -> adw::ToolbarView {
    let main_content_sidebar_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .build();

    let main_content_sidebar_scrolled_window = gtk::ScrolledWindow::builder()
        .child(&main_content_sidebar_box)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .hscrollbar_policy(PolicyType::Never)
        .build();

    let main_content_sidebar_toolbar = ToolbarView::builder()
        .content(&main_content_sidebar_scrolled_window)
        .top_bar_style(ToolbarStyle::Flat)
        .bottom_bar_style(ToolbarStyle::Flat)
        .build();

    main_content_sidebar_toolbar.add_top_bar(
        &HeaderBar::builder()
            .title_widget(&WindowTitle::builder().title(t!("application_name")).build())
            .show_title(true)
            .build(),
    );

    let pci_label = gtk::Label::new(Some("PCI_TEST"));
    let usb_label = gtk::Label::new(Some("USB_TEST"));
    main_content_sidebar_box.append(&pci_label);
    for button in pci_buttons {
        main_content_sidebar_box.append(button);
    }
    main_content_sidebar_box.append(&usb_label);
    for button in usb_buttons {
        main_content_sidebar_box.append(button);
    }

    main_content_sidebar_toolbar
}

fn custom_stack_selection_button(
    stack: &gtk::Stack,
    active: bool,
    name: String,
    title: String,
    icon_name: String,
    null_toggle_button: &gtk::ToggleButton,
) -> gtk::ToggleButton {
    let button_content = adw::ButtonContent::builder()
        .label(&title)
        .icon_name(icon_name)
        .halign(Align::Start)
        .build();
    let toggle_button = gtk::ToggleButton::builder()
        .group(null_toggle_button)
        .child(&button_content)
        .active(active)
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(10)
        .margin_end(10)
        .valign(gtk::Align::Start)
        .build();
    toggle_button.add_css_class("flat");
    toggle_button.connect_clicked(clone!(
        #[weak]
        stack,
        move |toggle_button| {
            if toggle_button.is_active() {
                stack.set_visible_child_name(&name);
            }
        }
    ));
    toggle_button
}

fn main_content_content(
    window: &adw::ApplicationWindow,
    window_banner: &adw::Banner,
    stack: &Stack,
    main_content_overlay_split_view: &adw::OverlaySplitView,
    window_breakpoint: &adw::Breakpoint,
) -> adw::ToolbarView {
    let window_headerbar = HeaderBar::builder()
        .title_widget(&WindowTitle::builder().title(t!("application_name")).build())
        .show_title(false)
        .build();
    let window_toolbar = ToolbarView::builder()
        .content(stack)
        .top_bar_style(ToolbarStyle::Flat)
        .bottom_bar_style(ToolbarStyle::Flat)
        .build();
    let sidebar_toggle_button = gtk::ToggleButton::builder()
        .icon_name("view-right-pane-symbolic")
        .visible(false)
        .build();
    let _sidebar_toggle_button_binding = main_content_overlay_split_view
        .bind_property("show_sidebar", &sidebar_toggle_button, "active")
        .sync_create()
        .bidirectional()
        .build();

    window_toolbar.add_top_bar(&window_headerbar);
    window_toolbar.add_top_bar(&window_banner.clone());
    window_breakpoint.add_setter(&sidebar_toggle_button, "visible", Some(&true.to_value()));
    window_breakpoint.add_setter(&window_headerbar, "show_title", Some(&true.to_value()));
    window_headerbar.pack_end(&sidebar_toggle_button);
    credits_window(&window, &window_headerbar);

    window_toolbar
}

fn credits_window(window: &adw::ApplicationWindow, window_headerbar: &adw::HeaderBar) {
    let credits_button = gtk::Button::builder()
        .icon_name("dialog-information-symbolic")
        .build();

    let credits_window = adw::AboutDialog::builder()
        .application_icon(APP_ICON)
        .application_name(t!("application_name"))
        .version(VERSION)
        .developer_name(t!("developer_name"))
        .license_type(gtk::License::Mpl20)
        .issue_url(APP_GIT.to_owned() + "/issues")
        .build();

    window_headerbar.pack_end(&credits_button);
    credits_button.connect_clicked(clone!(
        #[strong]
        window,
        move |_| credits_window.present(Some(&window))
    ));
}
fn internet_check_loop<F>(closure: F)
where
    F: FnOnce(bool) + 'static + Clone, // Closure takes `rx` as an argument
{
    let (sender, receiver) = async_channel::unbounded();

    thread::spawn(move || {
        let mut last_result = false;
        loop {
            if last_result == true {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }

            let check_internet_connection_cli = Command::new("ping")
                .arg("iso.pika-os.com")
                .arg("-c 1")
                .output()
                .expect("failed to execute process");
            if check_internet_connection_cli.status.success() {
                sender
                    .send_blocking(true)
                    .expect("The channel needs to be open.");
                last_result = true
            } else {
                sender
                    .send_blocking(false)
                    .expect("The channel needs to be open.");
                last_result = false
            }
        }
    });

    let main_context = MainContext::default();

    main_context.spawn_local(async move {
        while let Ok(state) = receiver.recv().await {
            let closure = closure.clone();
            closure(state);
        }
    });
}

fn create_pci_class(
    devices: &Vec<CfhdbPciDevice>,
    class: &str,
    theme_changed_action: &gio::SimpleAction,
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
        let device_title = format!("{} - {}", &device.vendor_name, &device.device_name);
        let device_navigation_page_toolbar = adw::ToolbarView::builder()
            .content(&pci_device_page(&device, &theme_changed_action, &device_status_indicator))
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

fn pci_device_page(device: &CfhdbPciDevice, theme_changed_action: &gio::SimpleAction, device_status_indicator: &ColoredCircle) -> gtk::Box {
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
    let started = device.started.unwrap_or_default();
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

    let device_id_color_badge = ColorBadge::new();
    device_id_color_badge.set_label0(textwrap::fill("TEST_DEVICE_ID", 10));
    device_id_color_badge.set_css_style("background-accent-bg");
    device_id_color_badge.set_group_size0(&color_badges_size_group0);
    device_id_color_badge.set_group_size1(&color_badges_size_group1);
    device_id_color_badge.set_theme_changed_action(theme_changed_action);

    color_badges_vec.push(&device_id_color_badge);
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
        .child(&gtk::Image::builder().icon_name("media-playback-start-symbolic").pixel_size(32).build())
        .width_request(48)
        .height_request(48)
        .tooltip_text("TEST_DEVICE_CONTROL_START")
        .build();
    let control_button_stop_device_button = gtk::Button::builder()
        .child(&gtk::Image::builder().icon_name("media-playback-stop-symbolic").pixel_size(32).build())
        .width_request(48)
        .height_request(48)
        .tooltip_text("TEST_DEVICE_CONTROL_STOP")
        .build();
    let control_button_enable_device_button = gtk::Button::builder()
        .child(&gtk::Image::builder().icon_name("emblem-ok-symbolic").pixel_size(32).build())
        .width_request(48)
        .height_request(48)
        .tooltip_text("TEST_DEVICE_CONTROL_ENABLE")
        .build();
    let control_button_disable_device_button = gtk::Button::builder()
        .child(&gtk::Image::builder().icon_name("edit-clear-all-symbolic").pixel_size(32).build())
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

    let profiles = device.available_profiles.0.lock().unwrap().clone().unwrap_or_default();
    let mut available_profiles_list_row_widgets = vec![];

    for profile in profiles {
        let (profiles_color_badges_size_group0 , profiles_color_badges_size_group1) = (gtk::SizeGroup::new(gtk::SizeGroupMode::Both), gtk::SizeGroup::new(gtk::SizeGroupMode::Both));
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
            available_profiles_list_row_widgets.push(profile_expander_row);
    }

    let update_device_status = || {
        let updated_device = CfhdbPciDevice::get_device_from_busid(&device.sysfs_busid).unwrap();
        let (started, enabled) = (updated_device.started.unwrap_or_default(), updated_device.enabled);
        let (color, tooltip) = match (enabled, started) {
            (true, true) => (RGBA::GREEN, "TEST_DEVICE_ACTIVE_ENABLED"),
            (false, true) => (RGBA::BLUE, "TEST_DEVICE_ACTIVE_DISABLED"),
            (true, false) => (RGBA::new(60.0, 255.0, 0.0, 1.0), "TEST_DEVICE_STOP_ENABLED"),
            (false, false) => (RGBA::RED, "TEST_DEVICE_STOP_DISABLED"),
        };
        device_status_indicator.set_color(color);
        device_status_indicator.set_tooltip_text(Some(tooltip));

        if started {
            control_button_start_device_button.set_sensitive(false);
        } else {
            control_button_stop_device_button.set_sensitive(true);
        }

        if enabled {
            control_button_enable_device_button.set_sensitive(false);
        } else {
            control_button_disable_device_button.set_sensitive(true);
        }

        started_color_badge.set_label1(textwrap::fill(
            if started {
                "TEST_YES"
            } else {
                "TEST_NO"
            },
            10,
        ));
        started_color_badge.set_css_style(if started {
            "background-accent-bg"
        } else {
            "background-red-bg"
        });
        enabled_color_badge.set_label1(textwrap::fill(
            if enabled {
                "TEST_YES"
            } else {
                "TEST_NO"
            },
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
        device_id_color_badge.set_label1(textwrap::fill(&device.device_id.as_str(), 10));
    };

    update_device_status();

    device_controls_box.append(&control_button_start_device_button);

    device_controls_box.append(&control_button_stop_device_button);

    device_controls_box.append(&control_button_enable_device_button);

    device_controls_box.append(&control_button_disable_device_button);

    content_box.append(&color_badges_grid);
    content_box.append(&device_controls_box);
    content_box.append(&available_profiles_list_row);

    content_box
}

fn theme_changed_thread(theme_changed_action: &gio::SimpleAction) {
    let (gsettings_change_sender, gsettings_change_receiver) = async_channel::unbounded();
    let gsettings_change_sender_clone0 = gsettings_change_sender.clone();

    thread::spawn(move || {
        let context = glib::MainContext::default();
        let main_loop = glib::MainLoop::new(Some(&context), false);
        let gsettings = gtk::gio::Settings::new("org.gnome.desktop.interface");
        gsettings.connect_changed(
            Some("accent-color"),
            clone!(
                #[strong]
                gsettings_change_sender_clone0,
                move |_, _| {
                    let gsettings_change_sender_clone0 = gsettings_change_sender_clone0.clone();
                    glib::timeout_add_seconds_local(5, move || {
                        gsettings_change_sender_clone0.send_blocking(()).unwrap();
                        glib::ControlFlow::Break
                    });
                }
            ),
        );
        main_loop.run()
    });

    let gsettings_changed_context = MainContext::default();
    // The main loop executes the asynchronous block
    gsettings_changed_context.spawn_local(clone!(
        #[strong]
        theme_changed_action,
        async move {
            while let Ok(()) = gsettings_change_receiver.recv().await {
                theme_changed_action.activate(None);
            }
        }
    ));
}
