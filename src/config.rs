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

/*pub fn distro_packages_installer(package_list: &str) {
    match duct::cmd!("pikman", "install", package_list).run() {
        Ok(_) => {
            println!(
                "[{}] {}",
                t!("info").bright_green(),
                t!("package_installation_successful")
            );
        }
        Err(_) => {
            eprintln!(
                "[{}] {}",
                t!("error").red(),
                t!("package_installation_failed")
            );
            exit(1);
        }
    }
}
pub fn distro_packages_uninstaller(package_list: &str) {
    match duct::cmd!("pikman", "purge", package_list).run() {
        Ok(_) => {
            match duct::cmd!("pikman", "purge").run() {
                Ok(_) => {
                    println!(
                        "[{}] {}",
                        t!("info").bright_green(),
                        t!("package_removal_successful")
                    );
                }
                Err(_) => {
                    eprintln!("[{}] {}", t!("error").red(), t!("package_removal_failed"));
                    exit(1);
                }
            }
            println!(
                "[{}] {}",
                t!("info").bright_green(),
                t!("package_removal_successful")
            );
        }
        Err(_) => {
            eprintln!("[{}] {}", t!("error").red(), t!("package_removal_failed"));
            exit(1);
        }
    }
}*/