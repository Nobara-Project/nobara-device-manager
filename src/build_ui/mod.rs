use crate::cfhdb::pci::{get_pci_devices, get_pci_profiles_from_url};
use crate::cfhdb::usb::{get_usb_devices, get_usb_profiles_from_url};
use crate::config::{APP_GIT, APP_ICON, APP_ID, VERSION};
use crate::ChannelMsg;
use adw::prelude::*;
use adw::*;
use gtk::glib::{clone, MainContext};
use gtk::{Align, Orientation, PolicyType, Stack, StackTransitionType, ToggleButton, Widget};
use gtk::Orientation::Vertical;
use libcfhdb::pci::CfhdbPciDevice;
use libcfhdb::usb::CfhdbUsbDevice;
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use gtk::ffi::GtkWidget;

pub fn build_ui(app: &adw::Application) {
    // setup glib
    gtk::glib::set_prgname(Some(t!("app_name").to_string()));
    glib::set_application_name(&t!("app_name").to_string());
    //
    let glib_settings = gio::Settings::new(APP_ID);

    // create the main Application window
    let window = adw::ApplicationWindow::builder()
        .title(t!("application_name"))
        .application(app)
        .icon_name(APP_ICON)
        .width_request(400)
        .height_request(650)
        .default_width(glib_settings.int("window-width"))
        .default_height(glib_settings.int("window-height"))
        .startup_id(APP_ID)
        .build();

    if glib_settings.boolean("is-maximized") == true {
        window.maximize()
    }

    window.connect_close_request(move |window| {
        if let Some(application) = window.application() {
            save_window_size(&window, &glib_settings);
            application.remove_window(window);
        }
        std::process::exit(0);
    });

    loading_content(&window);

    // show the window
    window.present()
}

fn loading_content(window: &ApplicationWindow) {
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

fn main_content(
    window: &adw::ApplicationWindow,
    hashmap_pci: HashMap<String, Vec<CfhdbPciDevice>>,
    hashmap_usb: HashMap<String, Vec<CfhdbUsbDevice>>,
) -> adw::OverlaySplitView {
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

    for (class, devices) in hashmap_pci {
        let class = format!("pci_class_name_{}", class);
        window_stack.add_titled(
            &gtk::Label::new(Some(&class)),
            Some(&class),
            &t!(class).to_string(),
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
            t!(class).to_string(),
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

    main_content_overlay_split_view.set_sidebar(Some(&main_content_sidebar(&window_stack, &pci_buttons, &usb_buttons)));

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

fn main_content_sidebar(stack: &gtk::Stack, pci_buttons: &Vec<ToggleButton>, usb_buttons: &Vec<ToggleButton>) -> adw::ToolbarView {
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

pub fn save_window_size(window: &adw::ApplicationWindow, glib_settings: &gio::Settings) {
    let size = window.default_size();

    let _ = glib_settings.set_int("window-width", size.0);
    let _ = glib_settings.set_int("window-height", size.1);
    let _ = glib_settings.set_boolean("is-maximized", window.is_maximized());
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
