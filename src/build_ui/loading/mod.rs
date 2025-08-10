use std::sync::Arc;
use std::time::Instant;

use crate::{
    build_ui::content::main_content,
    cfhdb::{
        bt::{get_bt_devices, get_bt_profiles_from_url, PreCheckedBtDevice, PreCheckedBtProfile},
        dmi::{get_dmi_info, get_dmi_profiles_from_url, PreCheckedDmiProfile},
        pci::{
            get_pci_devices, get_pci_profiles_from_url, PreCheckedPciDevice, PreCheckedPciProfile,
        },
        usb::{
            get_usb_devices, get_usb_profiles_from_url, PreCheckedUsbDevice, PreCheckedUsbProfile,
        },
    },
    config::APP_ICON,
    ChannelMsg,
};
use adw::{prelude::*, *};
use gtk::{
    glib::{clone, MainContext},
    Orientation,
};
use rayon;
use rayon::prelude::*;

macro_rules! rayon_join {
    ($a1:expr, $a2:expr) => {{
        rayon::join($a1, $a2)
    }};
    ($a1:expr, $a2:expr, $a3:expr) => {{
        let ((r1, r2), r3) = rayon::join(|| rayon_join!($a1, $a2), $a3);
        (r1, r2, r3)
    }};
    ($a1:expr, $a2:expr, $a3:expr, $a4:expr) => {{
        let ((r1, r2), (r3, r4)) = rayon::join(|| join!($a1, $a2), || join!($a3, $a4));
        (r1, r2, r3, r4)
    }};
    ($a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr) => {{
        let ((r1, r2, r3), (r4, r5)) = rayon::join(|| join!($a1, $a2, $a3), || join!($a4, $a5));
        (r1, r2, r3, r4, r5)
    }};
    ($a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr) => {{
        let ((r1, r2, r3), (r4, r5, r6)) =
            rayon::join(|| join!($a1, $a2, $a3), || join!($a4, $a5, $a6));
        (r1, r2, r3, r4, r5, r6)
    }};
    ($a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr) => {{
        let ((r1, r2, r3, r4), (r5, r6, r7)) =
            rayon::join(|| join!($a1, $a2, $a3, $a4), || join!($a5, $a6, $a7));
        (r1, r2, r3, r4, r5, r6, r7)
    }};
    ($a1:expr, $a2:expr, $a3:expr, $a4:expr, $a5:expr, $a6:expr, $a7:expr, $a8:expr) => {{
        let ((r1, r2, r3, r4), (r5, r6, r7, r8)) =
            rayon::join(|| join!($a1, $a2, $a3, $a4), || join!($a5, $a6, $a7, $a8));
        (r1, r2, r3, r4, r5, r6, r7, r8)
    }};
}

const PERM_FIX_PROG: &str = r###"
#! /bin/bash

USER=$(whoami)

chown $USER:$USER -R /var/cache/cfhdb || pkexec chown $USER:$USER -R /var/cache/cfhdb 
chmod 777 -R /var/cache/cfhdb || pkexec chmod 777 -R /var/cache/cfhdb
rm -rf /var/cache/cfhdb/check_cmd.sh || pkexec rm -rf /var/cache/cfhdb/check_cmd.sh

"###;

pub fn loading_content(
    window: &ApplicationWindow,
    about_action: &gio::SimpleAction,
    showallprofiles_action: &gio::SimpleAction,
) {
    let (status_sender, status_receiver) = async_channel::unbounded::<ChannelMsg>();
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
        about_action,
        #[strong]
        showallprofiles_action,
        async move {
            while let Ok(state) = status_receiver.recv().await {
                match state {
                    ChannelMsg::OutputLine(output_str) => {
                        loading_label.set_label(&output_str);
                    }
                    ChannelMsg::SuccessMsgDeviceFetch(
                        hashmap_pci,
                        hashmap_usb,
                        dmi_info,
                        hashmap_bt,
                        pci_profiles,
                        usb_profiles,
                        dmi_profiles,
                        bt_profiles,
                    ) => {
                        window.set_content(Some(&main_content(
                            &window,
                            hashmap_pci,
                            hashmap_usb,
                            dmi_info,
                            hashmap_bt,
                            pci_profiles,
                            usb_profiles,
                            dmi_profiles,
                            bt_profiles,
                            &about_action,
                            &showallprofiles_action,
                        )));
                    }
                    ChannelMsg::FailMsg => {}
                    ChannelMsg::SuccessMsg | ChannelMsg::UpdateMsg => {
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
        let total_start = Instant::now();

        // fix perms
        duct::cmd!("bash", "-c", PERM_FIX_PROG).run().unwrap();

        // Step 1: Download profiles
        status_sender
            .send_blocking(ChannelMsg::OutputLine(format!(
                "[{}] {}",
                t!("info"),
                t!("downloading_profiles")
            )))
            .expect("Channel closed");

        // Get DMI profiles
        let dmi_start = Instant::now();
        let dmi_profiles_result = get_dmi_profiles_from_url(&status_sender);
        let dmi_download_time = dmi_start.elapsed();
        println!("[PERF] DMI profiles download took: {:?}", dmi_download_time);

        // Get PCI profiles
        let pci_start = Instant::now();
        let pci_profiles_result = get_pci_profiles_from_url(&status_sender);
        let pci_download_time = pci_start.elapsed();
        println!("[PERF] PCI profiles download took: {:?}", pci_download_time);

        // Get USB profiles
        let usb_start = Instant::now();
        let usb_profiles_result = get_usb_profiles_from_url(&status_sender);
        let usb_download_time = usb_start.elapsed();
        println!("[PERF] USB profiles download took: {:?}", usb_download_time);

        // Get BT profiles
        let bt_start = Instant::now();
        let bt_profiles_result = get_bt_profiles_from_url(&status_sender);
        let bt_download_time = bt_start.elapsed();
        println!("[PERF] BT profiles download took: {:?}", bt_download_time);

        // Process DMI profiles
        let dmi_process_start = Instant::now();
        let dmi_profiles: Vec<Arc<PreCheckedDmiProfile>> = match dmi_profiles_result {
            Ok(t) => t
                .into_par_iter()
                .map(|x| {
                    let profile = PreCheckedDmiProfile::new(x);
                    profile.update_installed();
                    Arc::new(profile)
                })
                .collect(),
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
        let dmi_process_time = dmi_process_start.elapsed();
        println!(
            "[PERF] DMI profiles processing took: {:?}",
            dmi_process_time
        );

        // Process PCI profiles
        let pci_process_start = Instant::now();
        let pci_profiles: Vec<Arc<PreCheckedPciProfile>> = match pci_profiles_result {
            Ok(t) => t
                .into_par_iter()
                .map(|x| {
                    let profile = PreCheckedPciProfile::new(x);
                    profile.update_installed();
                    Arc::new(profile)
                })
                .collect(),
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
        let pci_process_time = pci_process_start.elapsed();
        println!(
            "[PERF] PCI profiles processing took: {:?}",
            pci_process_time
        );

        // Process USB profiles
        let usb_process_start = Instant::now();
        let usb_profiles: Vec<Arc<PreCheckedUsbProfile>> = match usb_profiles_result {
            Ok(t) => t
                .into_par_iter()
                .map(|x| {
                    let profile = PreCheckedUsbProfile::new(x);
                    profile.update_installed();
                    Arc::new(profile)
                })
                .collect(),
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
        let usb_process_time = usb_process_start.elapsed();
        println!(
            "[PERF] USB profiles processing took: {:?}",
            usb_process_time
        );

        // Process BT profiles
        let bt_process_start = Instant::now();
        let bt_profiles: Vec<Arc<PreCheckedBtProfile>> = match bt_profiles_result {
            Ok(t) => t
                .into_par_iter()
                .map(|x| {
                    let profile = PreCheckedBtProfile::new(x);
                    profile.update_installed();
                    Arc::new(profile)
                })
                .collect(),
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
        let bt_process_time = bt_process_start.elapsed();
        println!("[PERF] BT profiles processing took: {:?}", bt_process_time);

        // Step 2: Process devices
        status_sender
            .send_blocking(ChannelMsg::OutputLine(format!(
                "[{}] {}",
                t!("info"),
                t!("processing_devices")
            )))
            .expect("Channel closed");

        // Process DMI Info
        let dmi_info_start = Instant::now();
        let dmi_info = get_dmi_info(&dmi_profiles);
        let dmi_info_time = dmi_info_start.elapsed();
        println!("[PERF] DMI Info processing took: {:?}", dmi_info_time);

        // Process PCI devices
        let pci_devices_start = Instant::now();
        let hashmap_pci_result = get_pci_devices(pci_profiles.as_slice());
        let pci_devices_time = pci_devices_start.elapsed();
        println!("[PERF] PCI devices processing took: {:?}", pci_devices_time);

        // Process USB devices - this is likely the bottleneck
        let usb_devices_start = Instant::now();
        let hashmap_usb_result = get_usb_devices(usb_profiles.as_slice());
        let usb_devices_time = usb_devices_start.elapsed();
        println!("[PERF] USB devices processing took: {:?}", usb_devices_time);

        // Process BT devices
        let bt_devices_start = Instant::now();
        let hashmap_bt_result = get_bt_devices(bt_profiles.as_slice());
        let bt_devices_time = bt_devices_start.elapsed();
        println!("[PERF] BT devices processing took: {:?}", bt_devices_time);

        match (hashmap_pci_result, hashmap_usb_result, hashmap_bt_result) {
            (Some(hashmap_pci), Some(hashmap_usb), Some(hashmap_bt)) => {
                status_sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("info"),
                        t!("sorting_devices")
                    )))
                    .expect("Channel closed");

                // Convert to vectors
                let convert_start = Instant::now();
                let mut pci_vec: Vec<(String, Vec<PreCheckedPciDevice>)> =
                    hashmap_pci.into_iter().collect();
                let mut usb_vec: Vec<(String, Vec<PreCheckedUsbDevice>)> =
                    hashmap_usb.into_iter().collect();
                let mut bt_vec: Vec<(String, Vec<PreCheckedBtDevice>)> =
                    hashmap_bt.into_iter().collect();
                let convert_time = convert_start.elapsed();
                println!(
                    "[PERF] Converting hashmaps to vectors took: {:?}",
                    convert_time
                );

                // Sort the vectors in parallel
                let sort_start = Instant::now();
                rayon_join!(
                    || {
                        pci_vec.par_sort_by(|a, b| {
                            let a_class = t!(format!("pci_class_name_{}", a.0))
                                .to_string()
                                .to_lowercase();
                            let b_class = t!(format!("pci_class_name_{}", b.0))
                                .to_string()
                                .to_lowercase();
                            b_class.cmp(&a_class)
                        });
                    },
                    || {
                        usb_vec.par_sort_by(|a, b| {
                            let a_class = t!(format!("usb_class_name_{}", a.0))
                                .to_string()
                                .to_lowercase();
                            let b_class = t!(format!("usb_class_name_{}", b.0))
                                .to_string()
                                .to_lowercase();
                            b_class.cmp(&a_class)
                        });
                    },
                    || {
                        bt_vec.par_sort_by(|a, b| {
                            let a_class = t!(format!("bt_class_name_{}", a.0))
                                .to_string()
                                .to_lowercase();
                            let b_class = t!(format!("bt_class_name_{}", b.0))
                                .to_string()
                                .to_lowercase();
                            b_class.cmp(&a_class)
                        });
                    }
                );
                let sort_time = sort_start.elapsed();
                println!("[PERF] Sorting vectors took: {:?}", sort_time);

                let total_time = total_start.elapsed();
                println!("[PERF] Total loading time: {:?}", total_time);

                status_sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("info"),
                        t!("loading_complete")
                    )))
                    .expect("Channel closed");

                status_sender
                    .send_blocking(ChannelMsg::SuccessMsgDeviceFetch(
                        pci_vec,
                        usb_vec,
                        dmi_info,
                        bt_vec,
                        pci_profiles,
                        usb_profiles,
                        dmi_profiles,
                        bt_profiles,
                    ))
                    .expect("Channel closed");
            }
            (_, _, _) => {
                status_sender
                    .send_blocking(ChannelMsg::FailMsg)
                    .expect("Channel closed");
                panic!();
            }
        }
    });
}
