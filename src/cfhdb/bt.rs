use crate::{config::*, ChannelMsg};
use libcfhdb::bt::*;
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct PreCheckedBtDevice {
    pub device: CfhdbBtDevice,
    pub profiles: Vec<Arc<PreCheckedBtProfile>>,
}

pub struct PreCheckedBtProfile {
    profile: CfhdbBtProfile,
    installed: Arc<Mutex<bool>>,
}

impl PreCheckedBtProfile {
    pub fn new(profile: CfhdbBtProfile) -> Self {
        Self {
            profile,
            installed: Arc::new(Mutex::new(false)),
        }
    }
    pub fn profile(&self) -> CfhdbBtProfile {
        self.profile.clone()
    }
    pub fn installed(&self) -> bool {
        self.installed.lock().unwrap().clone()
    }
    pub fn update_installed(&self) {
        *self.installed.lock().unwrap() = self.profile.get_status();
    }
}

pub fn get_bt_devices(
    profiles: &[Arc<PreCheckedBtProfile>],
) -> Option<HashMap<String, Vec<PreCheckedBtDevice>>> {
    match CfhdbBtDevice::get_devices() {
        Some(devices) => {
            let hashmap = CfhdbBtDevice::create_class_hashmap(devices);
            return Some(
                hashmap
                    .iter()
                    .map(move |x| {
                        let mut pre_checked_devices = vec![];
                        for i in x.1 {
                            pre_checked_devices.push(get_pre_checked_device(profiles, i.clone()));
                        }
                        (x.0.clone(), pre_checked_devices)
                    })
                    .collect(),
            );
        }
        None => return None,
    }
}

fn get_pre_checked_device(
    profile_data: &[Arc<PreCheckedBtProfile>],
    device: CfhdbBtDevice,
) -> PreCheckedBtDevice {
    let mut available_profiles = vec![];
    for profile_arc in profile_data.iter() {
        let profile = profile_arc.profile();
        let matching = {
            if (profile.blacklisted_class_ids.contains(&"*".to_owned())
                || profile.blacklisted_class_ids.contains(&device.class_id))
                || (profile.blacklisted_bt_names.contains(&"*".to_owned())
                    || profile.blacklisted_bt_names.contains(&device.name))
                || (profile
                    .blacklisted_modalias_device_ids
                    .contains(&"*".to_owned())
                    || profile
                        .blacklisted_modalias_device_ids
                        .contains(&device.modalias_device_id))
                || (profile
                    .blacklisted_modalias_product_ids
                    .contains(&"*".to_owned())
                    || profile
                        .blacklisted_modalias_product_ids
                        .contains(&device.modalias_product_id))
                || (profile
                    .blacklisted_modalias_vendor_ids
                    .contains(&"*".to_owned())
                    || profile
                        .blacklisted_modalias_vendor_ids
                        .contains(&device.modalias_vendor_id))
            {
                false
            } else {
                let mut result = true;
                for (profile_field, info_field) in [
                    (&profile.bt_names, &device.name),
                    (&profile.modalias_device_ids, &device.modalias_device_id),
                    (&profile.modalias_product_ids, &device.modalias_product_id),
                    (&profile.modalias_vendor_ids, &device.modalias_vendor_id),
                ] {
                    if profile_field.contains(&"*".to_owned()) || profile_field.contains(info_field)
                    {
                        continue;
                    } else {
                        result = false;
                        break;
                    }
                }
                result
            }
        };

        if matching {
            available_profiles.push(profile_arc.clone());
        }
    }
    PreCheckedBtDevice {
        device,
        profiles: available_profiles,
    }
}

pub fn get_bt_profiles_from_url(
    sender: &async_channel::Sender<ChannelMsg>,
) -> Result<Vec<CfhdbBtProfile>, std::io::Error> {
    let cached_db_path = Path::new("/var/cache/cfhdb/bt.json");
    sender
        .send_blocking(ChannelMsg::OutputLine(format!(
            "[{}] {}",
            t!("info"),
            t!("bt_download_starting")
        )))
        .expect("Channel closed");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let data = match client.get(BT_PROFILE_JSON_URL.clone()).send() {
        Ok(t) => {
            sender
                .send_blocking(ChannelMsg::OutputLine(format!(
                    "[{}] {}",
                    t!("info"),
                    t!("bt_download_successful")
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
                    t!("bt_download_failed")
                )))
                .expect("Channel closed");
            if cached_db_path.exists() {
                sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("info"),
                        t!("bt_download_cache_found")
                    )))
                    .expect("Channel closed");
                fs::read_to_string(cached_db_path).unwrap()
            } else {
                sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("error"),
                        t!("bt_download_cache_not_found")
                    )))
                    .expect("Channel closed");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    t!("bt_download_cache_not_found"),
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
            let class_ids: Vec<String> = match profile["class_ids"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let bt_names: Vec<String> = match profile["bt_names"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let modalias_vendor_ids: Vec<String> = match profile["modalias_vendor_ids"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let modalias_device_ids: Vec<String> = match profile["modalias_device_ids"].as_array() {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let modalias_product_ids: Vec<String> = match profile["modalias_product_ids"].as_array()
            {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let blacklisted_class_ids: Vec<String> =
                match profile["blacklisted_class_ids"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let blacklisted_bt_names: Vec<String> = match profile["blacklisted_bt_names"].as_array()
            {
                Some(t) => t
                    .into_iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            };
            let blacklisted_modalias_vendor_ids: Vec<String> =
                match profile["blacklisted_modalias_vendor_ids"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let blacklisted_modalias_device_ids: Vec<String> =
                match profile["blacklisted_modalias_device_ids"].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
            let blacklisted_modalias_product_ids: Vec<String> =
                match profile["blacklisted_modalias_product_ids"].as_array() {
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
                        .expect("invalid_bt_profile_json")
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
            let veiled = profile["veiled"].as_bool().unwrap_or_default();
            let priority = profile["priority"].as_i64().unwrap_or_default();
            // Parse into the Struct
            let profile_struct = CfhdbBtProfile {
                codename,
                i18n_desc,
                icon_name,
                license,
                class_ids,
                bt_names,
                modalias_vendor_ids,
                modalias_device_ids,
                modalias_product_ids,
                blacklisted_class_ids,
                blacklisted_bt_names,
                blacklisted_modalias_vendor_ids,
                blacklisted_modalias_device_ids,
                blacklisted_modalias_product_ids,
                packages,
                check_script,
                install_script,
                remove_script,
                experimental,
                removable,
                veiled,
                priority: priority as i32,
            };
            profiles_array.push(profile_struct);
            profiles_array.sort_by_key(|x| x.priority);
        }
    }
    Ok(profiles_array)
}
