use crate::build_ui::content::main_content;
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
use users::get_current_username;
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use super::content::exec_duct_with_live_channel_stdout;

pub fn loading_content(window: &ApplicationWindow) {
    let (status_sender, status_receiver) = async_channel::unbounded::<ChannelMsg>();
    let device_maps = Arc::new((
        Mutex::new(None::<HashMap<String, CfhdbPciDevice>>),
        Mutex::new(None::<HashMap<String, CfhdbUsbDevice>>),
    ));
    let loading_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .vexpand(true)
        .hexpand(true)
        .build();
    let window_headerbar = HeaderBar::builder()
        .title_widget(&WindowTitle::builder().title(t!("application_name")).build())
        .build();
    let window_toolbar = ToolbarView::builder()
        .content(&loading_box)
        .top_bar_style(ToolbarStyle::Flat)
        .bottom_bar_style(ToolbarStyle::Flat)
        .build();

    let loading_icon = gtk::Image::builder()
        .icon_name(APP_ICON)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .vexpand(true)
        .hexpand(true)
        .margin_end(20)
        .pixel_size(256)
        .build();

    let loading_spinner = adw::Spinner::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .width_request(120)
        .height_request(120)
        .margin_end(20)
        .build();

    let loading_label = gtk::Label::builder()
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .vexpand(true)
        .hexpand(true)
        .margin_end(20)
        .build();

    let main_context = MainContext::default();

    main_context.spawn_local(clone!(
        #[weak]
        window,
        #[weak]
        loading_label,
        async move {
            while let Ok(state) = status_receiver.recv().await {
                match state {
                    ChannelMsg::OutputLine(output_str) => {
                        loading_label.set_label(&output_str);
                    }
                    ChannelMsg::SuccessMsgDeviceFetch(hashmap_pci, hashmap_usb) => {
                        window.set_content(Some(&main_content(&window, hashmap_pci, hashmap_usb)));
                    }
                    ChannelMsg::FailMsg => {}
                    ChannelMsg::SuccessMsg => {
                        panic!()
                    }
                }
            }
        }
    ));

    load_cfhdb(status_sender);

    window_toolbar.add_top_bar(&window_headerbar);
    loading_box.append(&loading_icon);
    loading_box.append(&loading_spinner);
    loading_box.append(&loading_label);

    window.set_content(Some(&window_toolbar));
}

fn load_cfhdb(status_sender: async_channel::Sender<ChannelMsg>) {
    std::thread::spawn(move || {
        let pci_profiles = match get_pci_profiles_from_url(&status_sender) {
            Ok(t) => t,
            Err(e) => {
                status_sender
                    .send_blocking(ChannelMsg::OutputLine(e.to_string()))
                    .expect("Channel closed");
                status_sender
                    .send_blocking(ChannelMsg::FailMsg)
                    .expect("Channel closed");
                panic!();
            }
        };
        let usb_profiles = match get_usb_profiles_from_url(&status_sender) {
            Ok(t) => t,
            Err(e) => {
                status_sender
                    .send_blocking(ChannelMsg::OutputLine(e.to_string()))
                    .expect("Channel closed");
                status_sender
                    .send_blocking(ChannelMsg::FailMsg)
                    .expect("Channel closed");
                panic!();
            }
        };
        match (
            get_pci_devices(&pci_profiles),
            get_usb_devices(&usb_profiles),
        ) {
            (Some(a), Some(b)) => {
                status_sender
                    .send_blocking(ChannelMsg::SuccessMsgDeviceFetch(a, b))
                    .expect("Channel closed");
            }
            (_, _) => {
                status_sender
                    .send_blocking(ChannelMsg::FailMsg)
                    .expect("Channel closed");
                panic!();
            }
        }
    });
}

pub fn run_in_lock_script(log_loop_sender: &async_channel::Sender<ChannelMsg>, script: &str) {
    let file_path = "/var/cache/cfhdb/script_lock.sh";
    let file_fs_path = std::path::Path::new(file_path);
    if file_fs_path.exists() {
        std::fs::remove_file(file_fs_path).unwrap();
    }
    {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .expect(&(file_path.to_string() + "cannot be read"));
        file.write_all(script.as_bytes())
            .expect(&(file_path.to_string() + "cannot be written to"));
        let mut perms = file
            .metadata()
            .expect(&(file_path.to_string() + "cannot be read"))
            .permissions();
        perms.set_mode(0o777);
        std::fs::set_permissions(file_path, perms)
            .expect(&(file_path.to_string() + "cannot be written to"));
    }
    let final_cmd = if get_current_username().unwrap() == "root" {
        duct::cmd!(file_path)
    } else {
        duct::cmd!("pkexec", file_path)
    };
    match exec_duct_with_live_channel_stdout(&log_loop_sender, final_cmd) {
        Ok(_) => {
            log_loop_sender
                .send_blocking(ChannelMsg::SuccessMsg)
                .unwrap();
        }
        Err(_) => {
            log_loop_sender.send_blocking(ChannelMsg::FailMsg).unwrap();
        }
    }
}
