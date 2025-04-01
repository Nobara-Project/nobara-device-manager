use std::sync::Arc;

use crate::{
    build_ui::content::main_content,
    cfhdb::{
        pci::{get_pci_devices, get_pci_profiles_from_url, PreCheckedPciDevice, PreCheckedPciProfile},
        usb::{get_usb_devices, get_usb_profiles_from_url, PreCheckedUsbDevice},
    },
    config::APP_ICON,
    ChannelMsg,
};
use adw::{prelude::*, *};
use gtk::{
    glib::{clone, MainContext},
    Orientation,
};
use libcfhdb::usb::CfhdbUsbDevice;

pub fn loading_content(window: &ApplicationWindow) {
    let (status_sender, status_receiver) = async_channel::unbounded::<ChannelMsg>();
    let (profile_update_sender, profile_update_receiver) = async_channel::unbounded::<ChannelMsg>();
    let update_device_status_action = gio::SimpleAction::new("update_device_status", None);
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
        #[strong]
        loading_label,
        #[strong]
        update_device_status_action,
        async move {
            while let Ok(state) = status_receiver.recv().await {
                match state {
                    ChannelMsg::OutputLine(output_str) => {
                        loading_label.set_label(&output_str);
                    }
                    ChannelMsg::SuccessMsgDeviceFetch(hashmap_pci, hashmap_usb) => {
                        window.set_content(Some(&main_content(&window, &update_device_status_action, hashmap_pci, hashmap_usb)));
                    }
                    ChannelMsg::FailMsg  => {}
                    ChannelMsg::SuccessMsg | ChannelMsg::UpdateMsg => {
                        panic!()
                    }
                }
            }
        }
    ));

    let profile_update_context = MainContext::default();

    profile_update_context.spawn_local(clone!(
        #[strong]
        update_device_status_action,
        async move {
            while let Ok(state) = profile_update_receiver.recv().await {
                match state {
                    ChannelMsg::UpdateMsg => {
                        update_device_status_action.activate(None);
                    }
                    _ => {
                        panic!()
                    }
                }
            }
        }
    ));

    load_cfhdb(status_sender, profile_update_sender);

    window_toolbar.add_top_bar(&window_headerbar);
    loading_box.append(&loading_icon);
    loading_box.append(&loading_spinner);
    loading_box.append(&loading_label);

    window.set_content(Some(&window_toolbar));
}

fn load_cfhdb(status_sender: async_channel::Sender<ChannelMsg>, profile_update_sender: async_channel::Sender<ChannelMsg>) {
    std::thread::spawn(move || {
        let pci_profiles: Vec<Arc<PreCheckedPciProfile>> = match get_pci_profiles_from_url(&status_sender) {
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
        }.into_iter().map(|x| {
            let profile = PreCheckedPciProfile::new(x, profile_update_sender.clone());
            profile.update_installed();
            Arc::new(profile)
        }).collect();
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
            get_pci_devices(pci_profiles.as_slice()),
            get_usb_devices(profile_update_sender, &usb_profiles),
        ) {
            (Some(hashmap_pci), Some(hashmap_usb)) => {
                let mut hashmap_pci: Vec<(String, Vec<PreCheckedPciDevice>)> = hashmap_pci.into_iter().collect();
                hashmap_pci.sort_by(|a, b| {
                    let a_class = t!(format!("pci_class_name_{}", a.0))
                        .to_string()
                        .to_lowercase();
                    let b_class = t!(format!("pci_class_name_{}", b.0))
                        .to_string()
                        .to_lowercase();
                    b_class.cmp(&a_class)
                });
                let mut hashmap_usb: Vec<(String, Vec<PreCheckedUsbDevice>)> = hashmap_usb.into_iter().collect();
                hashmap_usb.sort_by(|a, b| {
                    let a_class = t!(format!("usb_class_name_{}", a.0))
                        .to_string()
                        .to_lowercase();
                    let b_class = t!(format!("usb_class_name_{}", b.0))
                        .to_string()
                        .to_lowercase();
                    b_class.cmp(&a_class)
                });
                status_sender
                    .send_blocking(ChannelMsg::SuccessMsgDeviceFetch(hashmap_pci, hashmap_usb))
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
