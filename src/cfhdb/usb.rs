use crate::{config::*, ChannelMsg};
use libcfhdb::usb::*;
use std::{collections::HashMap, fs, path::Path};

pub fn get_usb_devices(
    profiles: &Vec<CfhdbUsbProfile>,
) -> Option<HashMap<String, Vec<CfhdbUsbDevice>>> {
    match CfhdbUsbDevice::get_devices() {
        Some(devices) => {
            for i in &devices {
                CfhdbUsbDevice::set_available_profiles(&profiles, &i);
            }
            let hashmap = CfhdbUsbDevice::create_class_hashmap(devices);
            return Some(hashmap);
        }
        None => return None,
    }
}
pub fn get_usb_profiles_from_url(
    sender: &async_channel::Sender<ChannelMsg>,
) -> Result<Vec<CfhdbUsbProfile>, std::io::Error> {
    let cached_db_path = Path::new("/var/cache/cfhdb/usb.json");
    sender
        .send_blocking(ChannelMsg::OutputLine(format!(
            "[{}] {}",
            t!("info"),
            t!("usb_download_starting")
        )))
        .expect("Channel closed");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let data = match client.get(USB_PROFILE_JSON_URL.clone()).send() {
        Ok(t) => {
            sender
                .send_blocking(ChannelMsg::OutputLine(format!(
                    "[{}] {}",
                    t!("info"),
                    t!("usb_download_successful")
                )))
                .expect("Channel closed");
            let cache = t.text().unwrap();
            let _ = fs::File::create(cached_db_path);
            let _ = fs::write(cached_db_path, &cache);
            cache
        }
        Err(_) => {
            sender
                .send_blocking(ChannelMsg::OutputLine(format!(
                    "[{}] {}",
                    t!("warn"),
                    t!("usb_download_failed")
                )))
                .expect("Channel closed");
            if cached_db_path.exists() {
                sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("info"),
                        t!("usb_download_cache_found")
                    )))
                    .expect("Channel closed");
                fs::read_to_string(cached_db_path).unwrap()
            } else {
                sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("error"),
                        t!("usb_download_cache_not_found")
                    )))
                    .expect("Channel closed");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    t!("usb_download_cache_not_found"),
                ));
            }
        }
    };
    let mut profiles_array = vec![];
    let res: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");
    if let serde_json::Value::Array(profiles) = &res["profiles"] {
        for profile in profiles {
            let codename = profile["codename"].as_str().unwrap_or_default().to_string();
            let i18n_desc =
                match profile[format!("i18n_desc[{}]", rust_i18n::locale().to_string())].as_str() {
                    Some(t) => {
                        if !t.is_empty() {
                            t.to_string()
                        } else {
                            profile["i18n_desc"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string()
                        }
                    }
                    None => profile["i18n_desc"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                };
            let icon_name = profile["icon_name"]
                .as_str()
                .unwrap_or("package-x-generic")
                .to_string();
            let license = profile["license"]
                .as_str()
                .unwrap_or(&t!("unknown"))
                .to_string();
            let class_codes: Vec<String> = match profile["class_codes"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let vendor_ids: Vec<String> = match profile["vendor_ids"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let product_ids: Vec<String> = match profile["product_ids"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let blacklisted_class_codes: Vec<String> =
                match profile["blacklisted_class_codes"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let blacklisted_vendor_ids: Vec<String> =
                match profile["blacklisted_vendor_ids"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let blacklisted_product_ids: Vec<String> =
                match profile["blacklisted_product_ids"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let packages: Option<Vec<String>> = match profile["packages"].as_str() {
                Some(_) => None,
                None => Some(
                    profile["packages"]
                        .as_array()
                        .expect("invalid_usb_profile_class_ids")
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                ),
            };
            let check_script = profile["check_script"]
                .as_str()
                .unwrap_or("false")
                .to_string();
            let install_script_value = profile["install_script"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let install_script = match install_script_value.as_str() {
                "Option::is_none" => None,
                _ => Some(install_script_value),
            };
            let remove_script_value = profile["remove_script"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let remove_script = match remove_script_value.as_str() {
                "Option::is_none" => None,
                _ => Some(remove_script_value),
            };
            let experimental = profile["experimental"].as_bool().unwrap_or_default();
            let removable = profile["removable"].as_bool().unwrap_or_default();
            let priority = profile["priority"].as_i64().unwrap_or_default();
            // Parse into the Struct
            let profile_struct = CfhdbUsbProfile {
                codename,
                i18n_desc,
                icon_name,
                license,
                class_codes,
                vendor_ids,
                product_ids,
                blacklisted_class_codes,
                blacklisted_vendor_ids,
                blacklisted_product_ids,
                packages,
                check_script,
                install_script,
                remove_script,
                experimental,
                removable,
                priority: priority as i32,
            };
            profiles_array.push(profile_struct);
            profiles_array.sort_by_key(|x| x.priority);
        }
    }
    Ok(profiles_array)
}
