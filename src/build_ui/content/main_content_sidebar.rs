use adw::{prelude::*, HeaderBar, ToolbarStyle, ToolbarView, WindowTitle};
use gtk::{Button, Orientation, PolicyType};

pub fn main_content_sidebar(
    pci_buttons: &Vec<Button>,
    usb_buttons: &Vec<Button>,
) -> adw::ToolbarView {
    let main_content_sidebar_box = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
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

    // PCI Devices Section
    let pci_label = gtk::Label::builder()
        .label(&t!("pci_devices").to_string())
        .halign(gtk::Align::Start)
        .margin_start(16)
        .margin_top(8)
        .margin_bottom(4)
        .build();
    pci_label.add_css_class("heading");
    
    main_content_sidebar_box.append(&pci_label);
    
    for button in pci_buttons {
        main_content_sidebar_box.append(button);
    }
    
    // Separator between sections
    let separator = gtk::Separator::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .build();
    main_content_sidebar_box.append(&separator);
    
    // USB Devices Section
    let usb_label = gtk::Label::builder()
        .label(&t!("usb_devices").to_string())
        .halign(gtk::Align::Start)
        .margin_start(16)
        .margin_top(8)
        .margin_bottom(4)
        .build();
    usb_label.add_css_class("heading");
    
    main_content_sidebar_box.append(&usb_label);
    
    for button in usb_buttons {
        main_content_sidebar_box.append(button);
    }

    main_content_sidebar_toolbar
}
