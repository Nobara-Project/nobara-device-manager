use std::sync::Arc;
use std::time::Instant;

use crate::{
    build_ui::content::main_content,
    cfhdb::{
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
use rayon::prelude::*;

pub fn loading_content(window: &ApplicationWindow) {
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
        async move {
            while let Ok(state) = status_receiver.recv().await {
                match state {
                    ChannelMsg::OutputLine(output_str) => {
                        loading_label.set_label(&output_str);
                    }
                    ChannelMsg::SuccessMsgDeviceFetch(hashmap_pci, hashmap_usb, pci_profiles, usb_profiles) => {
                        window.set_content(Some(&main_content(&window, hashmap_pci, hashmap_usb, pci_profiles, usb_profiles)));
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
        
        // Step 1: Download profiles
        status_sender
            .send_blocking(ChannelMsg::OutputLine(format!(
                "[{}] {}",
                t!("info"),
                t!("downloading_profiles")
            )))
            .expect("Channel closed");
            
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
        
        // Process PCI profiles
        let pci_process_start = Instant::now();
        let pci_profiles: Vec<Arc<PreCheckedPciProfile>> = match pci_profiles_result {
            Ok(t) => {
                t.into_par_iter()
                    .map(|x| {
                        let profile = PreCheckedPciProfile::new(x);
                        profile.update_installed();
                        Arc::new(profile)
                    })
                    .collect()
            },
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
        println!("[PERF] PCI profiles processing took: {:?}", pci_process_time);
        
        // Process USB profiles
        let usb_process_start = Instant::now();
        let usb_profiles: Vec<Arc<PreCheckedUsbProfile>> = match usb_profiles_result {
            Ok(t) => {
                t.into_par_iter()
                    .map(|x| {
                        let profile = PreCheckedUsbProfile::new(x);
                        profile.update_installed();
                        Arc::new(profile)
                    })
                    .collect()
            },
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
        println!("[PERF] USB profiles processing took: {:?}", usb_process_time);
            
        // Step 2: Process devices
        status_sender
            .send_blocking(ChannelMsg::OutputLine(format!(
                "[{}] {}",
                t!("info"),
                t!("processing_devices")
            )))
            .expect("Channel closed");
            
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
        
        match (hashmap_pci_result, hashmap_usb_result) {
            (Some(hashmap_pci), Some(hashmap_usb)) => {
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
                let convert_time = convert_start.elapsed();
                println!("[PERF] Converting hashmaps to vectors took: {:?}", convert_time);
                    
                // Sort the vectors in parallel
                let sort_start = Instant::now();
                rayon::join(
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
                    .send_blocking(ChannelMsg::SuccessMsgDeviceFetch(pci_vec, usb_vec, pci_profiles, usb_profiles))
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
