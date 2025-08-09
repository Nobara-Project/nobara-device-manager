use std::{rc::Rc, sync::Arc};

use adw::{prelude::*, AlertDialog};
use gtk::gio::SimpleAction;
use gtk::{glib::clone, Orientation};

use crate::cfhdb::dmi::PreCheckedDmiProfile;
use crate::{
    build_ui::color_badge::ColorBadge,
    cfhdb::{pci::PreCheckedPciProfile, usb::PreCheckedUsbProfile},
};

use super::{dmi, pci, usb};

pub fn all_profile_dialog(
    window: adw::ApplicationWindow,
    update_device_status_action: &SimpleAction,
    theme_changed_action: &SimpleAction,
    dmi_profiles: &Rc<Vec<Arc<PreCheckedDmiProfile>>>,
    pci_profiles: &Rc<Vec<Arc<PreCheckedPciProfile>>>,
    usb_profiles: &Rc<Vec<Arc<PreCheckedUsbProfile>>>,
) -> AlertDialog {
    let boxedlist = gtk::ListBox::builder()
        .vexpand(true)
        .hexpand(true)
        .margin_bottom(20)
        .margin_top(20)
        .margin_start(10)
        .margin_end(20)
        .build();
    boxedlist.add_css_class("boxed-list");
    let scroll = gtk::ScrolledWindow::builder()
        .width_request(600)
        .height_request(400)
        .propagate_natural_height(true)
        .propagate_natural_width(true)
        .vexpand(true)
        .hexpand(true)
        .child(&boxedlist)
        .build();
    let dialog = AlertDialog::builder()
        .extra_child(&scroll)
        .heading(t!(format!("all_profile_dialog_heading")))
        .build();
    dialog.add_response(
        "dialog_ok",
        &t!("profile_install_dialog_ok_label").to_string(),
    );
    let rows_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);
    //
    let dmi_profiles_clone0 = dmi_profiles.clone();
    let dmi_profiles_clone1: Vec<Arc<PreCheckedDmiProfile>> =
        dmi_profiles.iter().map(|f| f.clone()).collect();
    for profile in dmi_profiles_clone1 {
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
        let badges_warp_box = gtk::Box::new(Orientation::Vertical, 3);
        badges_warp_box.append(&color_badge_license);
        badges_warp_box.append(&color_badge_experimental);
        profile_content_row.add_prefix(&badges_warp_box);
        profile_action_box.append(&profile_remove_button);
        profile_action_box.append(&profile_install_button);
        profile_content_row.add_suffix(&profile_action_box);
        profile_expander_row.add_row(&profile_content_row);
        rows_size_group.add_widget(&profile_action_box);
        //
        profile_install_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_device_status_action,
            #[strong]
            profile,
            #[strong]
            dmi_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                dmi::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &dmi_profiles_clone0,
                    "install",
                    &theme_changed_action,
                );
            }
        ));
        profile_remove_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_device_status_action,
            #[strong]
            profile,
            #[strong]
            dmi_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                dmi::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &dmi_profiles_clone0,
                    "remove",
                    &theme_changed_action,
                );
            }
        ));
        //
        boxedlist.append(&profile_expander_row);
        //
        update_device_status_action.connect_activate(clone!(move |_, _| {
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
    //
    let pci_profiles_clone0 = pci_profiles.clone();
    let pci_profiles_clone1: Vec<Arc<PreCheckedPciProfile>> =
        pci_profiles.iter().map(|f| f.clone()).collect();
    for profile in pci_profiles_clone1 {
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
        let badges_warp_box = gtk::Box::new(Orientation::Vertical, 3);
        badges_warp_box.append(&color_badge_license);
        badges_warp_box.append(&color_badge_experimental);
        profile_content_row.add_prefix(&badges_warp_box);
        profile_action_box.append(&profile_remove_button);
        profile_action_box.append(&profile_install_button);
        profile_content_row.add_suffix(&profile_action_box);
        profile_expander_row.add_row(&profile_content_row);
        rows_size_group.add_widget(&profile_action_box);
        //
        profile_install_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_device_status_action,
            #[strong]
            profile,
            #[strong]
            pci_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                pci::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &pci_profiles_clone0,
                    "install",
                    &theme_changed_action,
                );
            }
        ));
        profile_remove_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_device_status_action,
            #[strong]
            profile,
            #[strong]
            pci_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                pci::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &pci_profiles_clone0,
                    "remove",
                    &theme_changed_action,
                );
            }
        ));
        //
        boxedlist.append(&profile_expander_row);
        //
        update_device_status_action.connect_activate(clone!(move |_, _| {
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
    //
    let usb_profiles_clone0 = usb_profiles.clone();
    let usb_profiles_clone1: Vec<Arc<PreCheckedUsbProfile>> =
        usb_profiles.iter().map(|f| f.clone()).collect();
    for profile in usb_profiles_clone1 {
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
        let badges_warp_box = gtk::Box::new(Orientation::Vertical, 3);
        badges_warp_box.append(&color_badge_license);
        badges_warp_box.append(&color_badge_experimental);
        profile_content_row.add_prefix(&badges_warp_box);
        profile_action_box.append(&profile_remove_button);
        profile_action_box.append(&profile_install_button);
        profile_content_row.add_suffix(&profile_action_box);
        profile_expander_row.add_row(&profile_content_row);
        rows_size_group.add_widget(&profile_action_box);
        //
        profile_install_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_device_status_action,
            #[strong]
            profile,
            #[strong]
            usb_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                usb::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &usb_profiles_clone0,
                    "install",
                    &theme_changed_action,
                );
            }
        ));
        profile_remove_button.connect_clicked(clone!(
            #[strong]
            window,
            #[strong]
            update_device_status_action,
            #[strong]
            profile,
            #[strong]
            usb_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                usb::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &usb_profiles_clone0,
                    "remove",
                    &theme_changed_action,
                );
            }
        ));
        //
        boxedlist.append(&profile_expander_row);
        //
        update_device_status_action.connect_activate(clone!(move |_, _| {
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
    dialog
}
