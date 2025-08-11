use adw::{prelude::*, HeaderBar, ToolbarStyle, ToolbarView, WindowTitle};
use gtk::{glib::clone, ListBox, ListBoxRow, Orientation, PolicyType};

pub fn main_content_sidebar(
    stack: &gtk::Stack,
    pci_rows: &Vec<ListBoxRow>,
    usb_rows: &Vec<ListBoxRow>,
    dmi_row: &ListBoxRow,
    bt_rows: &Vec<ListBoxRow>,
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

    //

    let dmi_rows_listbox = ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();
    dmi_rows_listbox.add_css_class("navigation-sidebar");

    let pci_rows_listbox = ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();
    pci_rows_listbox.add_css_class("navigation-sidebar");

    let usb_rows_listbox = ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();
    usb_rows_listbox.add_css_class("navigation-sidebar");

    let bt_rows_listbox = ListBox::builder()
        .selection_mode(gtk::SelectionMode::Single)
        .build();
    bt_rows_listbox.add_css_class("navigation-sidebar");

    // DMI Devices Section
    let dmi_label = gtk::Label::builder()
        .label(&t!("dmi_devices").to_string())
        .halign(gtk::Align::Start)
        .margin_start(16)
        .margin_top(8)
        .margin_bottom(4)
        .build();
    dmi_label.add_css_class("heading");

    main_content_sidebar_box.append(&dmi_label);
    main_content_sidebar_box.append(&dmi_rows_listbox);

    dmi_rows_listbox.append(dmi_row);
    dmi_rows_listbox.select_row(Some(dmi_row));

    dmi_rows_listbox.connect_row_activated(clone!(
        #[strong]
        pci_rows_listbox,
        #[strong]
        usb_rows_listbox,
        #[strong]
        bt_rows_listbox,
        #[strong]
        stack,
        move |_, row| {
            pci_rows_listbox.select_row(None::<&ListBoxRow>);
            usb_rows_listbox.select_row(None::<&ListBoxRow>);
            bt_rows_listbox.select_row(None::<&ListBoxRow>);
            stack.set_visible_child_name(&row.widget_name());
        }
    ));

    // Separator between sections
    let separator = gtk::Separator::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    // PCI Devices Section
    let pci_label = gtk::Label::builder()
        .label(&t!("pci_devices").to_string())
        .halign(gtk::Align::Start)
        .margin_start(16)
        .margin_top(8)
        .margin_bottom(4)
        .build();
    pci_label.add_css_class("heading");

    if !pci_rows.is_empty() {
        main_content_sidebar_box.append(&separator);
        main_content_sidebar_box.append(&pci_label);
        main_content_sidebar_box.append(&pci_rows_listbox);

        for row in pci_rows {
            pci_rows_listbox.append(row);
        }
    }

    pci_rows_listbox.connect_row_activated(clone!(
        #[strong]
        usb_rows_listbox,
        #[strong]
        dmi_rows_listbox,
        #[strong]
        bt_rows_listbox,
        #[strong]
        stack,
        move |_, row| {
            usb_rows_listbox.select_row(None::<&ListBoxRow>);
            dmi_rows_listbox.select_row(None::<&ListBoxRow>);
            bt_rows_listbox.select_row(None::<&ListBoxRow>);
            stack.set_visible_child_name(&row.widget_name());
        }
    ));

    // Separator between sections
    let separator = gtk::Separator::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    // USB Devices Section
    let usb_label = gtk::Label::builder()
        .label(&t!("usb_devices").to_string())
        .halign(gtk::Align::Start)
        .margin_start(16)
        .margin_top(8)
        .margin_bottom(4)
        .build();
    usb_label.add_css_class("heading");

    if !usb_rows.is_empty() {
        main_content_sidebar_box.append(&separator);
        main_content_sidebar_box.append(&usb_label);
        main_content_sidebar_box.append(&usb_rows_listbox);
        for row in usb_rows {
            usb_rows_listbox.append(row);
        }
    }

    usb_rows_listbox.connect_row_activated(clone!(
        #[strong]
        pci_rows_listbox,
        #[strong]
        dmi_rows_listbox,
        #[strong]
        bt_rows_listbox,
        #[strong]
        stack,
        move |_, row| {
            pci_rows_listbox.select_row(None::<&ListBoxRow>);
            dmi_rows_listbox.select_row(None::<&ListBoxRow>);
            bt_rows_listbox.select_row(None::<&ListBoxRow>);
            stack.set_visible_child_name(&row.widget_name());
        }
    ));

    // Separator between sections
    let separator = gtk::Separator::builder()
        .orientation(Orientation::Horizontal)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    // BT Devices Section
    let bt_label = gtk::Label::builder()
        .label(&t!("bt_devices").to_string())
        .halign(gtk::Align::Start)
        .margin_start(16)
        .margin_top(8)
        .margin_bottom(4)
        .build();
    bt_label.add_css_class("heading");

    if !bt_rows.is_empty() {
        main_content_sidebar_box.append(&separator);
        main_content_sidebar_box.append(&bt_label);
        main_content_sidebar_box.append(&bt_rows_listbox);
        for row in bt_rows {
            bt_rows_listbox.append(row);
        }
    }

    bt_rows_listbox.connect_row_activated(clone!(
        #[strong]
        pci_rows_listbox,
        #[strong]
        usb_rows_listbox,
        #[strong]
        dmi_rows_listbox,
        #[strong]
        stack,
        move |_, row| {
            pci_rows_listbox.select_row(None::<&ListBoxRow>);
            dmi_rows_listbox.select_row(None::<&ListBoxRow>);
            usb_rows_listbox.select_row(None::<&ListBoxRow>);
            stack.set_visible_child_name(&row.widget_name());
        }
    ));

    main_content_sidebar_toolbar
}
