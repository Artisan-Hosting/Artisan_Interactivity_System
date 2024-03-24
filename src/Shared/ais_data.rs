use std::{
    fs::File, io::{Read, Write}, path::PathBuf,
};

use if_addrs::get_if_addrs;
use mac_address::get_mac_address;
use serde::{Deserialize, Serialize};
use system::path_present;
use crate::errors::{AisError, UnifiedError};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AisInfo {
    pub pages_id: Option<String>,
    pub client_id: Option<String>,
    pub machine_id: Option<String>,
    pub machine_mac: Option<String>,
    pub machine_ip: Option<String>,
    pub ssh_events: usize,
}

impl AisInfo {
    pub fn new() -> Self {
        let file_location = "/etc/artisan.manifest";
        let manifest_data = Self::fetch_manifest(file_location);
         
        let data: AisInfo = AisInfo {
            pages_id: manifest_data
                .get("pages_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            client_id: manifest_data
                .get("client_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            machine_id: manifest_data
                .get("machine_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            machine_mac: Self::fetch_machine_mac(),
            machine_ip: Self::fetch_machine_ip(),
            ssh_events: 0,
        };

        return data
    }

    pub fn print_all(&self) {
        if let Some(client_id) = &self.client_id {
            println!("Client ID: {:?}", client_id);
        }
        if let Some(machine_id) = &self.machine_id {
            println!("Machine ID: {:?}", machine_id);
        }
        if let Some(machine_mac) = &self.machine_mac {
            println!("Machine MAC: {}", machine_mac);
        }
        if let Some(machine_ip) = &self.machine_ip {
            println!("Machine IP: {}", machine_ip);
        }
    }

    fn fetch_manifest(file_location: &str) -> serde_json::Value {
        if let Ok(true) = path_present(PathBuf::from(file_location)) {
            if let Ok(mut file) = File::open(file_location) {
                let mut file_contents = String::new();
                if let Err(_) = file.read_to_string(&mut file_contents) {
                    return serde_json::Value::Object(Default::default());
                }
                if let Ok(json_data) = serde_json::from_str(&file_contents) {
                    return json_data;
                }
            }
        }
        serde_json::Value::Object(Default::default())
    }

    pub fn create_manifest(&self, file_location: &str) -> Result<(), UnifiedError> {
        // Serialize AisInfo to JSON
        let json_data = serde_json::to_string(self).map_err(|e| UnifiedError::AisError(AisError::new(&e.to_string())))?;

        // Write the JSON data to the file
        let mut file = File::create(file_location).map_err(|e| UnifiedError::AisError(AisError::new(&e.to_string())))?;
        file.write_all(json_data.as_bytes()).map_err(|e| UnifiedError::AisError(AisError::new(&e.to_string())))?;

        Ok(())
    }

    fn fetch_machine_mac() -> Option<String> {
        match get_mac_address() {
            Ok(Some(mac)) => Some(mac.to_string()),
            _ => None,
        }
    }

    fn fetch_machine_ip() -> Option<String> {
        if let Ok(ifaces) = get_if_addrs() {
            for iface in ifaces {
                if iface.is_loopback() || !iface.ip().is_ipv4() {
                    continue;
                }
                return Some(iface.ip().to_string());
            }
        }
        None
    }
}
