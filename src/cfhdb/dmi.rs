use crate::{config::*, ChannelMsg};
use libcfhdb::dmi::*;
use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct PreCheckedDmiInfo {
    pub info: CfhdbDmiInfo,
    pub profiles: Vec<Arc<PreCheckedDmiProfile>>,
}

pub struct PreCheckedDmiProfile {
    profile: CfhdbDmiProfile,
    installed: Arc<Mutex<bool>>,
    pub used: Arc<Mutex<bool>>,
}

impl PreCheckedDmiProfile {
    pub fn new(profile: CfhdbDmiProfile) -> Self {
        Self {
            profile,
            installed: Arc::new(Mutex::new(false)),
            used: Arc::new(Mutex::new(false))
        }
    }
    pub fn profile(&self) -> CfhdbDmiProfile {
        self.profile.clone()
    }
    pub fn installed(&self) -> bool {
        self.installed.lock().unwrap().clone()
    }
    pub fn update_installed(&self) {
        *self.installed.lock().unwrap() = self.profile.get_status();
    }
}

pub fn get_dmi_info(profiles: &[Arc<PreCheckedDmiProfile>]) -> PreCheckedDmiInfo {
    let info = CfhdbDmiInfo::get_dmi();
    get_pre_checked_info(profiles, info.clone())
}

fn get_pre_checked_info(
    profile_data: &[Arc<PreCheckedDmiProfile>],
    info: CfhdbDmiInfo,
) -> PreCheckedDmiInfo {
    let mut available_profiles = vec![];
    for profile_arc in profile_data.iter() {
        let profile = profile_arc.profile();
        let matching = {
            if
            // BIOS
            profile.blacklisted_bios_vendors.contains(&"*".to_owned())
                    || profile.blacklisted_bios_vendors.contains(&info.bios_vendor)
                    // BOARD
                    || profile.blacklisted_board_asset_tags.contains(&"*".to_owned())
                    || profile.blacklisted_board_asset_tags.contains(&info.board_asset_tag)
                    || profile.blacklisted_board_names.contains(&"*".to_owned())
                    || profile.blacklisted_board_names.contains(&info.board_name)
                    || profile.blacklisted_board_vendors.contains(&"*".to_owned())
                    || profile.blacklisted_board_vendors.contains(&info.board_vendor)
                    // PRODUCT
                    || profile.blacklisted_product_families.contains(&"*".to_owned())
                    || profile.blacklisted_product_families.contains(&info.product_family)
                    || profile.blacklisted_product_names.contains(&"*".to_owned())
                    || profile.blacklisted_product_names.contains(&info.product_name)
                    || profile.blacklisted_product_skus.contains(&"*".to_owned())
                    || profile.blacklisted_product_skus.contains(&info.product_sku)
                    // Sys
                    || profile.blacklisted_sys_vendors.contains(&"*".to_owned())
                    || profile.blacklisted_sys_vendors.contains(&info.sys_vendor)
            {
                false
            } else {
                let mut result = true;
                for (profile_field, info_field) in [
                    (&profile.bios_vendors, &info.bios_vendor),
                    (&profile.board_asset_tags, &info.board_asset_tag),
                    (&profile.board_names, &info.board_name),
                    (&profile.board_vendors, &info.board_vendor),
                    (&profile.product_families, &info.product_family),
                    (&profile.product_names, &info.product_name),
                    (&profile.product_skus, &info.product_sku),
                    (&profile.sys_vendors, &info.sys_vendor),
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
    PreCheckedDmiInfo {
        info,
        profiles: available_profiles,
    }
}

pub fn get_dmi_profiles_from_url(
    sender: &async_channel::Sender<ChannelMsg>,
) -> Result<Vec<CfhdbDmiProfile>, std::io::Error> {
    let cached_db_path = Path::new("/var/cache/cfhdb/dmi.json");
    sender
        .send_blocking(ChannelMsg::OutputLine(format!(
            "[{}] {}",
            t!("info"),
            t!("dmi_download_starting")
        )))
        .expect("Channel closed");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let data = match client.get(DMI_PROFILE_JSON_URL.clone()).send() {
        Ok(t) => {
            sender
                .send_blocking(ChannelMsg::OutputLine(format!(
                    "[{}] {}",
                    t!("info"),
                    t!("dmi_download_successful")
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
                    t!("dmi_download_failed")
                )))
                .expect("Channel closed");
            if cached_db_path.exists() {
                sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("info"),
                        t!("dmi_download_cache_found")
                    )))
                    .expect("Channel closed");
                fs::read_to_string(cached_db_path).unwrap()
            } else {
                sender
                    .send_blocking(ChannelMsg::OutputLine(format!(
                        "[{}] {}",
                        t!("error"),
                        t!("dmi_download_cache_not_found")
                    )))
                    .expect("Channel closed");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    t!("dmi_download_cache_not_found"),
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
            let mut dmi_strings_vec = Vec::new();
            for dmi_string in [
                "bios_vendors",
                "board_asset_tags",
                "board_names",
                "board_vendors",
                "product_families",
                "product_names",
                "product_skus",
                "sys_vendors",
                "blacklisted_bios_vendors",
                "blacklisted_board_asset_tags",
                "blacklisted_board_names",
                "blacklisted_board_vendors",
                "blacklisted_product_families",
                "blacklisted_product_names",
                "blacklisted_product_skus",
                "blacklisted_sys_vendors",
            ] {
                let final_map: Vec<String> = match profile[dmi_string].as_array() {
                    Some(t) => t
                        .into_iter()
                        .map(|x| x.as_str().unwrap_or_default().to_string())
                        .collect(),
                    None => vec![],
                };
                dmi_strings_vec.push(Arc::new(final_map));
            }
            let packages: Option<Vec<String>> = match profile["packages"].as_str() {
                Some(_) => None,
                None => Some(
                    profile["packages"]
                        .as_array()
                        .expect("invalid_dmi_profile_json")
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
            let profile_struct = CfhdbDmiProfile {
                codename,
                i18n_desc,
                icon_name,
                license,
                bios_vendors: dmi_strings_vec[0].to_vec(),
                board_asset_tags: dmi_strings_vec[1].to_vec(),
                board_names: dmi_strings_vec[2].to_vec(),
                board_vendors: dmi_strings_vec[3].to_vec(),
                product_families: dmi_strings_vec[4].to_vec(),
                product_names: dmi_strings_vec[5].to_vec(),
                product_skus: dmi_strings_vec[6].to_vec(),
                sys_vendors: dmi_strings_vec[7].to_vec(),
                blacklisted_bios_vendors: dmi_strings_vec[8].to_vec(),
                blacklisted_board_asset_tags: dmi_strings_vec[9].to_vec(),
                blacklisted_board_names: dmi_strings_vec[10].to_vec(),
                blacklisted_board_vendors: dmi_strings_vec[11].to_vec(),
                blacklisted_product_families: dmi_strings_vec[12].to_vec(),
                blacklisted_product_names: dmi_strings_vec[13].to_vec(),
                blacklisted_product_skus: dmi_strings_vec[14].to_vec(),
                blacklisted_sys_vendors: dmi_strings_vec[15].to_vec(),
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
