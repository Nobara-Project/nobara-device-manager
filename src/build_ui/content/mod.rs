use crate::cfhdb::pci::{get_pci_devices, get_pci_profiles_from_url};
use crate::cfhdb::usb::{get_usb_devices, get_usb_profiles_from_url};
use crate::config::{APP_GIT, APP_ICON, APP_ID, VERSION};
use crate::ChannelMsg;
use adw::prelude::*;
use adw::*;
use duct::cmd;
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
use pci::create_pci_class;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use super::color_badge::ColorBadge;
use super::colored_circle::{self, ColoredCircle};

mod pci;

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
            &create_pci_class(&window, &devices, &class_i18n, &theme_changed_action),
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

pub fn exec_duct_with_live_channel_stdout(
    //sender: async_channel::Sender<String>,
    sender: &async_channel::Sender<ChannelMsg>,
    duct_expr: duct::Expression,
) -> Result<(), std::boxed::Box<dyn std::error::Error + Send + Sync>> {
    let (pipe_reader, pipe_writer) = os_pipe::pipe()?;
    let child = duct_expr
        .stderr_to_stdout()
        .stdout_file(pipe_writer)
        .start()?;
    for line in BufReader::new(pipe_reader).lines() {
        sender
            .send_blocking(ChannelMsg::OutputLine(line?))
            .expect("Channel needs to be opened.")
    }
    child.wait()?;

    Ok(())
}
