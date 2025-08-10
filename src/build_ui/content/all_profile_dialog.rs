use std::{rc::Rc, sync::Arc};

use adw::{prelude::*, AlertDialog};
use gtk::gio::SimpleAction;
use gtk::CheckButton;
use gtk::{glib::clone, Orientation};

use crate::cfhdb::bt::PreCheckedBtProfile;
use crate::cfhdb::dmi::PreCheckedDmiProfile;
use crate::{
    build_ui::color_badge::ColorBadge,
    cfhdb::{pci::PreCheckedPciProfile, usb::PreCheckedUsbProfile},
};

use super::{bt, dmi, pci, usb};

pub fn all_profile_dialog(
    window: adw::ApplicationWindow,
    update_device_status_action: &SimpleAction,
    theme_changed_action: &SimpleAction,
    dmi_profiles: &Rc<Vec<Arc<PreCheckedDmiProfile>>>,
    pci_profiles: &Rc<Vec<Arc<PreCheckedPciProfile>>>,
    usb_profiles: &Rc<Vec<Arc<PreCheckedUsbProfile>>>,
    bt_profiles: &Rc<Vec<Arc<PreCheckedBtProfile>>>,
) -> AlertDialog {
    let dialog_child_box = gtk::Box::new(Orientation::Vertical, 0);
    let hide_noninstalled_profiles_checkbutton = gtk::CheckButton::builder()
        .label(t!("hide_noninstalled_profiles_checkbutton_label"))
        .build();
    let hide_noncompatible_profiles_checkbutton = gtk::CheckButton::builder()
        .label(t!("hide_noncompatible_profiles_checkbutton_label"))
        .build();
    let profiles_checkbutton_box = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(5)
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(5)
        .margin_end(5)
        .build();
    profiles_checkbutton_box.append(&hide_noninstalled_profiles_checkbutton);
    profiles_checkbutton_box.append(&hide_noncompatible_profiles_checkbutton);
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
    dialog_child_box.append(&profiles_checkbutton_box);
    dialog_child_box.append(&scroll);
    let dialog = AlertDialog::builder()
        .extra_child(&dialog_child_box)
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
        let recheck_hide_closure = clone!(#[strong] profile_expander_row, #[strong] profile, #[strong] hide_noncompatible_profiles_checkbutton, #[strong] hide_noninstalled_profiles_checkbutton, move |_: &CheckButton| {
            profile_expander_row.set_visible(recheck_hide(hide_noncompatible_profiles_checkbutton.is_active(), hide_noninstalled_profiles_checkbutton.is_active(), *profile.used.lock().unwrap(), profile.installed()));
        });
        hide_noninstalled_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        hide_noncompatible_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        //
        boxedlist.append(&profile_expander_row);
        //
        update_device_status_action.connect_activate(clone!(#[strong] hide_noninstalled_profiles_checkbutton, move |_, _| {
            let profile_status = profile.installed();
            profile_install_button.set_sensitive(!profile_status);
            if profile_content.removable {
                profile_remove_button.set_sensitive(profile_status);
            } else {
                profile_remove_button.set_sensitive(false);
            }
            profile_status_icon.set_visible(profile_status);
            hide_noninstalled_profiles_checkbutton.emit_by_name::<()>("toggled", &[]);
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
        let recheck_hide_closure = clone!(#[strong] profile_expander_row, #[strong] profile, #[strong] hide_noncompatible_profiles_checkbutton, #[strong] hide_noninstalled_profiles_checkbutton, move |_: &CheckButton| {
            profile_expander_row.set_visible(recheck_hide(hide_noncompatible_profiles_checkbutton.is_active(), hide_noninstalled_profiles_checkbutton.is_active(), *profile.used.lock().unwrap(), profile.installed()));
        });
        hide_noninstalled_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        hide_noncompatible_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        //
        boxedlist.append(&profile_expander_row);
        //
        update_device_status_action.connect_activate(clone!(#[strong] hide_noninstalled_profiles_checkbutton, move |_, _| {
            let profile_status = profile.installed();
            profile_install_button.set_sensitive(!profile_status);
            if profile_content.removable {
                profile_remove_button.set_sensitive(profile_status);
            } else {
                profile_remove_button.set_sensitive(false);
            }
            profile_status_icon.set_visible(profile_status);
            hide_noninstalled_profiles_checkbutton.emit_by_name::<()>("toggled", &[]);
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
        let recheck_hide_closure = clone!(#[strong] profile_expander_row, #[strong] profile, #[strong] hide_noncompatible_profiles_checkbutton, #[strong] hide_noninstalled_profiles_checkbutton, move |_: &CheckButton| {
            profile_expander_row.set_visible(recheck_hide(hide_noncompatible_profiles_checkbutton.is_active(), hide_noninstalled_profiles_checkbutton.is_active(), *profile.used.lock().unwrap(), profile.installed()));
        });
        hide_noninstalled_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        hide_noncompatible_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        //
        boxedlist.append(&profile_expander_row);
        //
        update_device_status_action.connect_activate(clone!(#[strong] hide_noninstalled_profiles_checkbutton, move |_, _| {
            let profile_status = profile.installed();
            profile_install_button.set_sensitive(!profile_status);
            if profile_content.removable {
                profile_remove_button.set_sensitive(profile_status);
            } else {
                profile_remove_button.set_sensitive(false);
            }
            profile_status_icon.set_visible(profile_status);
            hide_noninstalled_profiles_checkbutton.emit_by_name::<()>("toggled", &[]);
        }));
    }
    //
    let bt_profiles_clone0 = bt_profiles.clone();
    let bt_profiles_clone1: Vec<Arc<PreCheckedBtProfile>> =
        bt_profiles.iter().map(|f| f.clone()).collect();
    for profile in bt_profiles_clone1 {
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
            bt_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                bt::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &bt_profiles_clone0,
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
            bt_profiles_clone0,
            #[strong]
            theme_changed_action,
            move |_| {
                bt::profile_modify(
                    window.clone(),
                    &update_device_status_action,
                    &profile,
                    &bt_profiles_clone0,
                    "remove",
                    &theme_changed_action,
                );
            }
        ));
        //
        let recheck_hide_closure = clone!(#[strong] profile_expander_row, #[strong] profile, #[strong] hide_noncompatible_profiles_checkbutton, #[strong] hide_noninstalled_profiles_checkbutton, move |_: &CheckButton| {
            profile_expander_row.set_visible(recheck_hide(hide_noncompatible_profiles_checkbutton.is_active(), hide_noninstalled_profiles_checkbutton.is_active(), *profile.used.lock().unwrap(), profile.installed()));
        });
        hide_noninstalled_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        hide_noncompatible_profiles_checkbutton.connect_toggled(recheck_hide_closure.clone());
        //
        boxedlist.append(&profile_expander_row);
        //
        update_device_status_action.connect_activate(clone!(#[strong] hide_noninstalled_profiles_checkbutton, move |_, _| {
            let profile_status = profile.installed();
            profile_install_button.set_sensitive(!profile_status);
            if profile_content.removable {
                profile_remove_button.set_sensitive(profile_status);
            } else {
                profile_remove_button.set_sensitive(false);
            }
            profile_status_icon.set_visible(profile_status);
            hide_noninstalled_profiles_checkbutton.emit_by_name::<()>("toggled", &[]);
        }));
    }
    dialog
}

fn recheck_hide(only_comapt: bool, only_install: bool, is_compat: bool, is_installed: bool) -> bool {
    if only_comapt && only_install {
        // Only show if BOTH compatible AND installed
        is_compat && is_installed
    } else if only_comapt {
        // Show if compatible
        is_compat
    } else if only_install {
        // Show if installed
        is_installed
    } else {
        // Default: show if compatible
        true
    }
}