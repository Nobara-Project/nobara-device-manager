use std::{rc::Rc, sync::Arc};

use adw::prelude::*;
use adw::{Banner, BreakpointCondition};
use gio::SimpleAction;
use gtk::glib::{self, clone};
use gtk::*;
use gtk::{Align, StackTransitionType, ToggleButton};

use crate::cfhdb::pci::{PreCheckedPciDevice, PreCheckedPciProfile};
use crate::cfhdb::usb::{PreCheckedUsbDevice, PreCheckedUsbProfile};

mod all_profile_dialog;
mod internet_check;
mod main_content_content;
mod main_content_sidebar;
mod pci;
mod usb;

use all_profile_dialog::all_profile_dialog;
use internet_check::internet_check_loop;
use main_content_content::{error_dialog, main_content_content};
use main_content_sidebar::main_content_sidebar;
use pci::create_pci_class;
use usb::create_usb_class;

pub fn main_content(
    window: &adw::ApplicationWindow,
    hashmap_pci: Vec<(String, Vec<PreCheckedPciDevice>)>,
    hashmap_usb: Vec<(String, Vec<PreCheckedUsbDevice>)>,
    pci_profiles: Vec<Arc<PreCheckedPciProfile>>,
    usb_profiles: Vec<Arc<PreCheckedUsbProfile>>,
    about_action: &gtk::gio::SimpleAction,
    showallprofiles_action: &gtk::gio::SimpleAction,
) -> adw::OverlaySplitView {
    // Start timing the UI building process
    let ui_start = std::time::Instant::now();

    let theme_changed_action = gio::SimpleAction::new("theme_changed", None);

    let window_breakpoint = adw::Breakpoint::new(BreakpointCondition::new_length(
        adw::BreakpointConditionLengthType::MaxWidth,
        900.0,
        adw::LengthUnit::Sp,
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

    let all_profiles_button = gtk::Button::builder()
        .icon_name("emblem-system-symbolic")
        .tooltip_text(t!("all_profiles_button_label"))
        .build();

    // Create a proper toggle button for the sidebar
    let sidebar_toggle = ToggleButton::builder()
        .icon_name("sidebar-show-symbolic")
        .tooltip_text(t!("toggle_sidebar"))
        .active(true)
        .build();
    
    // Connect the toggle button to the sidebar's collapsed state
    sidebar_toggle.connect_toggled(clone!(
        #[weak]
        main_content_overlay_split_view,
        move |toggle| {
            main_content_overlay_split_view.set_show_sidebar(toggle.is_active());
        }
    ));
    
    let update_device_status_action = gio::SimpleAction::new("update_device_status", None);

    let mut pci_rows = vec![];
    let mut usb_rows = vec![];

    let pci_profiles_rc = Rc::new(pci_profiles);
    let usb_profiles_rc = Rc::new(usb_profiles);
    let dialog = all_profile_dialog(
        window.clone(),
        &update_device_status_action,
        &theme_changed_action,
        &pci_profiles_rc,
        &usb_profiles_rc,
    );
    all_profiles_button.connect_clicked(clone!(
        #[strong]
        window,
        #[strong]
        dialog,
        move |_| {
            dialog.present(Some(&window));
        }
    ));
    showallprofiles_action.connect_activate(clone!(#[strong] all_profiles_button, move |_, _| {
        all_profiles_button.emit_clicked();
    }));

    theme_changed_thread(&theme_changed_action);

    // Create placeholder pages for each class
    for (class, devices) in hashmap_pci {
        let class = format!("pci_class_name_{}", class);
        let class_i18n = t!(class).to_string();

        // Create a placeholder page with a loading spinner
        let placeholder = create_placeholder_page(&class_i18n);

        window_stack.add_titled(&placeholder, Some(&class), &class_i18n);

        // Store the devices for lazy loading
        let devices_clone = devices.clone();
        let window_clone = window.clone();
        let theme_changed_action_clone = theme_changed_action.clone();
        let update_device_status_action_clone = update_device_status_action.clone();
        let class_i18n_clone = class_i18n.clone();

        // Connect to the "map" signal to load content when page becomes visible
        placeholder.connect_map(move |placeholder| {
            // Check if this page has already been loaded
            if let Some(child) = placeholder.first_child() {
                if child.widget_name() == "content_loaded" {
                    return;
                }
            }

            // Create the actual content
            let content = create_pci_class(
                &window_clone,
                &devices_clone,
                &class_i18n_clone,
                &theme_changed_action_clone,
                &update_device_status_action_clone,
            );
            content.set_widget_name("content_loaded");

            // Replace the placeholder with the actual content
            while let Some(child) = placeholder.first_child() {
                placeholder.remove(&child);
            }
            placeholder.append(&content);
        });

        pci_rows.push(custom_stack_selection_button(
            class.clone(),
            class_i18n,
            get_icon_for_class(&class)
                .unwrap_or("dialog-question-symbolic")
                .into(),
        ));
    }

    for (class, devices) in hashmap_usb {
        let class = format!("usb_class_name_{}", class);
        let class_i18n = t!(class).to_string();

        // Create a placeholder page with a loading spinner
        let placeholder = create_placeholder_page(&class_i18n);

        window_stack.add_titled(&placeholder, Some(&class), &class_i18n);

        // Store the devices for lazy loading
        let devices_clone = devices.clone();
        let window_clone = window.clone();
        let theme_changed_action_clone = theme_changed_action.clone();
        let update_device_status_action_clone = update_device_status_action.clone();
        let class_i18n_clone = class_i18n.clone();

        // Connect to the "map" signal to load content when page becomes visible
        placeholder.connect_map(move |placeholder| {
            // Check if this page has already been loaded
            if let Some(child) = placeholder.first_child() {
                if child.widget_name() == "content_loaded" {
                    return;
                }
            }

            // Create the actual content
            let content = create_usb_class(
                &window_clone,
                &devices_clone,
                &class_i18n_clone,
                &theme_changed_action_clone,
                &update_device_status_action_clone,
            );
            content.set_widget_name("content_loaded");

            // Replace the placeholder with the actual content
            while let Some(child) = placeholder.first_child() {
                placeholder.remove(&child);
            }
            placeholder.append(&content);
        });

        usb_rows.push(custom_stack_selection_button(
            class.clone(),
            class_i18n,
            get_icon_for_class(&class)
                .unwrap_or("dialog-question-symbolic")
                .into(),
        ));
    }

    main_content_overlay_split_view.set_content(Some(&main_content_content(
        &window,
        &window_banner,
        &window_stack,
        &main_content_overlay_split_view,
        &window_breakpoint,
        all_profiles_button.clone(),
        sidebar_toggle.clone(),
        &about_action
    )));

    main_content_overlay_split_view
        .set_sidebar(Some(&main_content_sidebar(&window_stack, &pci_rows, &usb_rows)));

    window_breakpoint.add_setter(
        &main_content_overlay_split_view,
        "collapsed",
        Some(&true.to_value()),
    );
    
    window_breakpoint.add_setter(
        &sidebar_toggle,
        "active",
        Some(&false.to_value()),
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

    // Print the time taken to build the UI
    println!("[PERF] Initial UI building took: {:?}", ui_start.elapsed());

    main_content_overlay_split_view
}

// Helper function to create a placeholder page with a loading spinner
fn create_placeholder_page(title: &str) -> gtk::Box {
    let box_container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .vexpand(true)
        .hexpand(true)
        .build();

    let spinner = gtk::Spinner::builder()
        .width_request(32)
        .height_request(32)
        .halign(Align::Center)
        .valign(Align::Center)
        .vexpand(true)
        .build();
    spinner.start();

    let label = gtk::Label::builder()
        .label(&format!("{} - {}", t!("loading"), title))
        .halign(Align::Center)
        .build();

    box_container.append(&spinner);
    box_container.append(&label);

    box_container
}

fn theme_changed_thread(theme_changed_action: &SimpleAction) {
    let (theme_change_sender, theme_change_receiver) = async_channel::unbounded();
    let theme_change_sender_clone = theme_change_sender.clone();

    std::thread::spawn(move || {
        let mut last_theme = "".to_string();
        loop {
            let current_theme = match std::fs::read_to_string("/tmp/cfhdb/theme") {
                Ok(t) => t,
                Err(_) => "".to_string(),
            };
            if current_theme != last_theme {
                last_theme = current_theme;
                let _ = theme_change_sender_clone.send_blocking(());
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    let theme_changed_context = glib::MainContext::default();
    // The main loop executes the asynchronous block
    theme_changed_context.spawn_local(clone!(
        #[strong]
        theme_changed_action,
        async move {
            while let Ok(()) = theme_change_receiver.recv().await {
                theme_changed_action.activate(None);
            }
        }
    ));
}

pub fn get_icon_for_class(class: &str) -> Option<&'static str> {
    match class {
        // pci_classes
        "pci_class_name_0000" => Some("dialog-question-symbolic"),
        "pci_class_name_0001" => Some("dialog-question-symbolic"),
        "pci_class_name_0100" => Some("scsi-symbolic"),
        "pci_class_name_0101" => Some("drive-harddisk-ieee1394-symbolic"),
        "pci_class_name_0102" => Some("media-floppy-symbolic"),
        "pci_class_name_0103" => Some("scsi-symbolic"),
        "pci_class_name_0104" => Some("path-combine-symbolic"),
        "pci_class_name_0105" => Some("drive-harddisk-ieee1394-symbolic"),
        "pci_class_name_0106" => Some("drive-harddisk-solidstate-symbolic"),
        "pci_class_name_0107" => Some("drive-harddisk-symbolic"),
        "pci_class_name_0108" => Some("drive-harddisk-solidstate-symbolic"),
        "pci_class_name_0109" => Some("media-flash-symbolic"),
        "pci_class_name_0180" => Some("drive-removable-media-symbolic"),
        "pci_class_name_0200" => Some("network-wired-symbolic"),
        "pci_class_name_0201" => Some("network-wired-symbolic"),
        "pci_class_name_0202" => Some("network-wired-symbolic"),
        "pci_class_name_0203" => Some("network-transmit-receive-symbolic"),
        "pci_class_name_0204" => Some("network-wired-symbolic"),
        "pci_class_name_0205" => Some("network-receive-symbolic"),
        "pci_class_name_0206" => Some("emblem-shared-symbolic"),
        "pci_class_name_0207" => Some("nvidia"),
        "pci_class_name_0208" => Some("intel"),
        "pci_class_name_0280" => Some("network-wired-symbolic"),
        "pci_class_name_0300" => Some("video-display-symbolic"),
        "pci_class_name_0301" => Some("camera-video-symbolic"),
        "pci_class_name_0302" => Some("applications-graphics-symbolic"),
        "pci_class_name_0380" => Some("video-display-symbolic"),
        "pci_class_name_0400" => Some("video-display-symbolic"),
        "pci_class_name_0401" => Some("audio-card-symbolic"),
        "pci_class_name_0402" => Some("call-start-symbolic"),
        "pci_class_name_0403" => Some("audio-card-symbolic"),
        "pci_class_name_0480" => Some("audio-card-symbolic"),
        "pci_class_name_0500" => Some("ram-symbolic"),
        "pci_class_name_0501" => Some("media-flash-symbolic"),
        "pci_class_name_0580" => Some("ram-symbolic"),
        "pci_class_name_0600" => Some("cpu-symbolic"),
        "pci_class_name_0601" => Some("cpu-symbolic"),
        "pci_class_name_0602" => Some("cpu-symbolic"),
        "pci_class_name_0603" => Some("cpu-symbolic"),
        "pci_class_name_0604" => Some("cpu-symbolic"),
        "pci_class_name_0605" => Some("cpu-symbolic"),
        "pci_class_name_0606" => Some("cpu-symbolic"),
        "pci_class_name_0607" => Some("cpu-symbolic"),
        "pci_class_name_0608" => Some("cpu-symbolic"),
        "pci_class_name_0609" => Some("cpu-symbolic"),
        "pci_class_name_060A" => Some("nvidia"),
        "pci_class_name_060B" => Some("cpu-symbolic"),
        "pci_class_name_0680" => Some("cpu-symbolic"),
        "pci_class_name_0700" => Some("serial-port-symbolic"),
        "pci_class_name_0701" => Some("format-justify-left-symbolic"),
        "pci_class_name_0702" => Some("serial-port-symbolic"),
        "pci_class_name_0703" => Some("modem-symbolic"),
        "pci_class_name_0704" => Some("modem-symbolic"),
        "pci_class_name_0705" => Some("auth-smartcard-symbolic"),
        "pci_class_name_0780" => Some("modem-symbolic"),
        "pci_class_name_0800" => Some("cpu-symbolic"),
        "pci_class_name_0801" => Some("cpu-symbolic"),
        "pci_class_name_0802" => Some("cpu-symbolic"),
        "pci_class_name_0803" => Some("cpu-symbolic"),
        "pci_class_name_0804" => Some("cpu-symbolic"),
        "pci_class_name_0805" => Some("cpu-symbolic"),
        "pci_class_name_0806" => Some("applications-utilities-symbolic"),
        "pci_class_name_0807" => Some("cpu-symbolic"),
        "pci_class_name_0880" => Some("cpu-symbolic"),
        "pci_class_name_0900" => Some("input-keyboard-symbolic"),
        "pci_class_name_0901" => Some("tool-pencil-symbolic"),
        "pci_class_name_0902" => Some("input-mouse-symbolic"),
        "pci_class_name_0903" => Some("scanner-symbolic"),
        "pci_class_name_0904" => Some("input-gaming-symbolic"),
        "pci_class_name_0980" => Some("input-keyboard-symbolic"),
        "pci_class_name_0A00" => Some("standard-input-symbolic"),
        "pci_class_name_0A80" => Some("standard-input-symbolic"),
        "pci_class_name_0B00" => Some("cpu-symbolic"),
        "pci_class_name_0B01" => Some("cpu-symbolic"),
        "pci_class_name_0B02" => Some("cpu-symbolic"),
        "pci_class_name_0B10" => Some("cpu-symbolic"),
        "pci_class_name_0B20" => Some("cpu-symbolic"),
        "pci_class_name_0B30" => Some("cpu-symbolic"),
        "pci_class_name_0B40" => Some("cpu-symbolic"),
        "pci_class_name_0B80" => Some("cpu-symbolic"),
        "pci_class_name_0C00" => Some("serial-port-symbolic"),
        "pci_class_name_0C01" => Some("serial-port-symbolic"),
        "pci_class_name_0C02" => Some("serial-port-symbolic"),
        "pci_class_name_0C03" => Some("drive-harddisk-usb-symbolic"),
        "pci_class_name_0C04" => Some("network-wired-symbolic"),
        "pci_class_name_0C05" => Some("cpu-symbolic"),
        "pci_class_name_0C06" => Some("nvidia"),
        "pci_class_name_0C07" => Some("serial-port-symbolic"),
        "pci_class_name_0C08" => Some("serial-port-symbolic"),
        "pci_class_name_0C09" => Some("serial-port-symbolic"),
        "pci_class_name_0C0A" => Some("serial-port-symbolic"),
        "pci_class_name_0C80" => Some("drive-harddisk-usb-symbolic"),
        "pci_class_name_0D00" => Some("input-tvremote-symbolic"),
        "pci_class_name_0D01" => Some("input-tvremote-symbolic"),
        "pci_class_name_0D10" => Some("input-tvremote-symbolic"),
        "pci_class_name_0D11" => Some("bluetooth-active-symbolic"),
        "pci_class_name_0D12" => Some("daytime-sunrise-symbolic"),
        "pci_class_name_0D20" => Some("network-cellular-edge-symbolic"),
        "pci_class_name_0D21" => Some("network-cellular-gprs-symbolic"),
        "pci_class_name_0D40" => Some("network-cellular-signal-good-symbolic"),
        "pci_class_name_0D41" => Some("network-cellular-signal-good-symbolic"),
        "pci_class_name_0D80" => Some("network-wireless-symbolic"),
        "pci_class_name_0E00" => Some("network-receive-symbolic"),
        "pci_class_name_0F01" => Some("daytime-sunrise-symbolic"),
        "pci_class_name_0F02" => Some("daytime-sunrise-symbolic"),
        "pci_class_name_0F03" => Some("daytime-sunrise-symbolic"),
        "pci_class_name_0F04" => Some("daytime-sunrise-symbolic"),
        "pci_class_name_0F80" => Some("daytime-sunrise-symbolic"),
        "pci_class_name_1000" => Some("network-wireless-encrypted-symbolic"),
        "pci_class_name_1010" => Some("network-wireless-encrypted-symbolic"),
        "pci_class_name_1080" => Some("network-wireless-encrypted-symbolic"),
        "pci_class_name_1100" => Some("cpu-symbolic"),
        "pci_class_name_1101" => Some("cpu-symbolic"),
        "pci_class_name_1110" => Some("cpu-symbolic"),
        "pci_class_name_1120" => Some("cpu-symbolic"),
        "pci_class_name_1180" => Some("cpu-symbolic"),
        "pci_class_name_1200" => Some("cpu-symbolic"),
        "pci_class_name_1300" => Some("cpu-symbolic"),
        // usb classes
        "usb_class_name_00" => Some("dialog-question-symbolic"),
        "usb_class_name_01" => Some("audio-card-symbolic"),
        "usb_class_name_02" => Some("network-wireless-symbolic"),
        "usb_class_name_03" => Some("input-keyboard-symbolic"),
        "usb_class_name_05" => Some("input-touchpad-symbolic"),
        "usb_class_name_06" => Some("camera-web-symbolic"),
        "usb_class_name_07" => Some("printer-symbolic"),
        "usb_class_name_08" => Some("drive-removable-media-symbolic"),
        "usb_class_name_09" => Some("media-playlist-shuffle-symbolic"),
        "usb_class_name_0A" => Some("drive-harddisk-usb-symbolic"),
        "usb_class_name_0B" => Some("auth-smartcard-symbolic"),
        "usb_class_name_0D" => Some("network-wireless-encrypted-symbolic"),
        "usb_class_name_0E" => Some("video-x-generic-symbolic"),
        "usb_class_name_0F" => Some("face-smile-big-symbolic"),
        "usb_class_name_10" => Some("video-x-generic-symbolic"),
        "usb_class_name_11" => Some("input-tablet-symbolic"),
        "usb_class_name_12" => Some("drive-harddisk-usb-symbolic"),
        "usb_class_name_13" => Some("video-display-symbolic"),
        "usb_class_name_14" => Some("drive-harddisk-usb-symbolic"),
        "usb_class_name_3C" => Some("cpu-symbolic"),
        "usb_class_name_DC" => Some("dialog-warning-symbolic"),
        "usb_class_name_E0" => Some("network-wireless-symbolic"),
        "usb_class_name_EF" => Some("dialog-question-symbolic"),
        "usb_class_name_FE" => Some("dialog-question-symbolic"),
        "usb_class_name_FF" => Some("dialog-question-symbolic"),
        //
        _ => None,
    }
}

fn custom_stack_selection_button(
    name: String,
    title: String,
    icon: String,
) -> gtk::ListBoxRow {
    // Create a box to hold the icon and label with proper spacing
    let button_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(8)
        .margin_end(8)
        .spacing(8)
        .build();
    
    // Add the icon
    let icon_widget = gtk::Image::builder()
        .icon_name(&icon)
        .pixel_size(16)
        .build();
    
    // Add the label
    let label = gtk::Label::builder()
        .label(&title)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();
    
    // Add them to the box
    button_box.append(&icon_widget);
    button_box.append(&label);
    
    // Create the button with the box as its child
    let listboxrow = gtk::ListBoxRow::builder()
        .child(&button_box)
        .tooltip_text(&title)
        .name(name)
        .build();
    
    listboxrow
}

pub fn run_in_lock_script(
    log_loop_sender: &async_channel::Sender<crate::ChannelMsg>,
    script: &str,
) {
    use crate::ChannelMsg;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use users::get_current_username;

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
    let username = get_current_username().unwrap();
    let final_cmd = if username == "root" {
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

pub fn exec_duct_with_live_channel_stdout(
    sender: &async_channel::Sender<crate::ChannelMsg>,
    duct_expr: duct::Expression,
) -> Result<(), std::boxed::Box<dyn std::error::Error + Send + Sync>> {
    use crate::ChannelMsg;
    use std::io::BufRead;

    let (pipe_reader, pipe_writer) = os_pipe::pipe()?;
    let child = duct_expr
        .stderr_to_stdout()
        .stdout_file(pipe_writer)
        .start()?;
    for line in std::io::BufReader::new(pipe_reader).lines() {
        sender
            .send_blocking(ChannelMsg::OutputLine(line?))
            .expect("Channel needs to be opened.")
    }

    child.wait()?;

    Ok(())
}
