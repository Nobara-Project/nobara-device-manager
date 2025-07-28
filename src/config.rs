// GTK

pub const APP_ID: &str = "com.github.pikaos-linux.pikadevicemanager";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_ICON: &str = "com.github.pikaos-linux.pikadevicemanager";
pub const APP_GIT: &str = "https://git.pika-os.com/custom-gui-packages/pika-device-manager";

// CFHDB

#[derive(serde::Deserialize)]
pub struct ProfileUrlConfig {
    pci_json_url: String,
    usb_json_url: String,
}

lazy_static::lazy_static! {
    pub static ref PCI_PROFILE_JSON_URL: String = get_profile_url_config().pci_json_url;
    pub static ref USB_PROFILE_JSON_URL: String = get_profile_url_config().usb_json_url;
}

fn get_profile_url_config() -> ProfileUrlConfig {
    let file_path = "/etc/cfhdb/profile-config.json";
    let json_content = std::fs::read_to_string(file_path).unwrap();
    let config: ProfileUrlConfig = serde_json::from_str(&json_content).unwrap();
    config
}
