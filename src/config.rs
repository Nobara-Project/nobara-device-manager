use std::process::exit;

// GTK

pub const APP_ID: &str = "com.github.pikaos-linux.pikadevicemanager";
pub const DISTRO_ICON: &str = "com.github.pikaos-linux.pikadevicemanager";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ICON: &str = "com.github.pikaos-linux.pikadevicemanager";
pub const APP_GIT: &str = "https://git.pika-os.com/custom-gui-packages/pika-device-manager";

// CFHDB

pub const PCI_PROFILE_JSON_URL: &str =
    "https://github.com/CosmicFusion/cfhdb/raw/refs/heads/master/data/profiles/pci.json";
pub const USB_PROFILE_JSON_URL: &str =
    "https://github.com/CosmicFusion/cfhdb/raw/refs/heads/master/data/profiles/usb.json";
