use std::{
    fmt,
    fs::File,
    io::{Read, Write},
};

use crate::errors::{AisError, UnifiedError};
use if_addrs::get_if_addrs;
use mac_address::get_mac_address;
use serde::{Deserialize, Serialize};
use system::{path_present, PathType};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AisInfo {
    pub pages_id: Option<String>,
    pub client_id: Option<String>,
    pub machine_id: Option<String>,
    pub machine_mac: Option<String>,
    pub machine_ip: Option<String>,
    pub ssh_events: usize,
    pub system_version: AisVersion,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AisVersion {
    pub version_number: f64,
    pub version_code: AisCode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AisCode {
    Production,
    ProductionCandidate,
    Beta,
    Alpha,
}

impl fmt::Display for AisCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ais_code = match self {
            AisCode::Production => "P",
            AisCode::ProductionCandidate => "RC",
            AisCode::Beta => "b",
            AisCode::Alpha => "a",
        };
        write!(f, "{}", ais_code)
    }
}

impl AisInfo {
    pub fn new() -> Result<Self, UnifiedError> {
        let manifest_data = Self::fetch_manifest()?;

        let ais_version: AisVersion = AisVersion {
            version_number: 1.10,
            version_code: AisCode::ProductionCandidate,
        };

        let data: AisInfo = AisInfo {
            pages_id: manifest_data.get("pages_id").and_then(|v| v.as_str().map(|s| s.to_string())),
            client_id: manifest_data
                .get("client_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            machine_id: manifest_data
                .get("machine_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            machine_mac: Self::fetch_machine_mac(),
            machine_ip: Self::fetch_machine_ip(),
            ssh_events: 0,
            system_version: ais_version,
        };

        return Ok(data);
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

    pub fn fetch_manifest() -> Result<serde_json::Value, UnifiedError> {
        let manifest_path: PathType = Self::fetch_manifest_path();
        match path_present(&manifest_path) {
            Ok(b) => {
                match b {
                    true => {
                        let mut file: File = match File::open(manifest_path) {
                            Ok(d) => d,
                            Err(_) => todo!(),
                        };
        
                        let mut buffer: Vec<u8> = Vec::new();
                        if let Err(e) = file.read_to_end(&mut buffer) {
                            return Err(UnifiedError::from_ais_error(AisError::new(&e.to_string())));
                        }
        
                        let manifest_data: serde_json::Value = match serde_json::from_slice(&buffer) {
                            Ok(d) => d,
                            Err(e) => {
                                return Err(UnifiedError::from_ais_error(AisError::new(&e.to_string())))
                            }
                        };
        
                        return Ok(manifest_data);
                    },
                    false => {
                        // We'll create a generic AisInfo to allow the firstrun to finish and auto generate
                        let generic_ais: AisInfo = AisInfo {
                            pages_id: None,
                            client_id: None,
                            machine_id: None,
                            machine_mac: Self::fetch_machine_mac(),
                            machine_ip: Self::fetch_machine_ip(),
                            ssh_events: 0,
                            system_version: AisVersion { version_number: 0.00, version_code: AisCode::Alpha },
                        };

                        let manifest_data: serde_json::Value = match serde_json::to_value(&generic_ais) {
                            Ok(d) => d,
                            Err(e) => {
                                return Err(UnifiedError::from_ais_error(AisError::new(&e.to_string())))
                            }
                        };
        
                        return Ok(manifest_data);
                    }
                }

            }
            Err(e) => return Err(UnifiedError::from_ais_error(AisError::new(&e.to_string()))),
        }
    }

    pub fn fetch_manifest_path() -> PathType {
        PathType::Str("/etc/artisan.manifest".into())
    }

    pub fn create_manifest(&self) -> Result<(), UnifiedError> {
        // Serialize AisInfo to JSON
        let json_data = serde_json::to_string(self)
            .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;

        // Write the JSON data to the file
        let mut file = File::create(Self::fetch_manifest_path())
            .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;
        file.write_all(json_data.as_bytes())
            .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;

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
