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

/// Struct representing information about the Ais system.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AisInfo {
    /// Unique identifier for pages.
    pub pages_id: Option<String>,
    /// Unique identifier for the client.
    pub client_id: Option<String>,
    /// Unique identifier for the machine.
    pub machine_id: Option<String>,
    /// MAC address of the machine.
    pub machine_mac: Option<String>,
    /// IP address of the machine.
    pub machine_ip: Option<String>,
    /// Number of SSH events.
    pub ssh_events: usize,
    /// Version information of the system.
    pub system_version: AisVersion,
}

/// Version information structure.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct AisVersion {
    /// Version number.
    pub version_number: f64,
    /// Version code.
    pub version_code: AisCode,
}

/// Enumeration representing different version codes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AisCode {
    /// Production version.
    Production,
    /// Production candidate version.
    ProductionCandidate,
    /// Beta version.
    Beta,
    /// Alpha version.
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
    /// Creates a new instance of `AisInfo`.
    pub fn new() -> Result<Self, UnifiedError> {
        let manifest_data = Self::fetch_manifest()?;


        let ais_version: AisVersion = match serde_json::from_value(manifest_data.get("system_version").unwrap().clone()) {
            Ok(d) => d,
            Err(_) => Self::current_version(),
        };

        Ok(AisInfo {
            pages_id: manifest_data
                .get("pages_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            client_id: manifest_data
                .get("client_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            machine_id: manifest_data
                .get("machine_id")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            machine_mac: manifest_data
                .get("machine_mac")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            machine_ip: manifest_data
                .get("machine_ip")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            ssh_events: 0,
            system_version: ais_version,
        })
    }

    /// Prints all available information.
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

    pub fn current_version() -> AisVersion {
        let new_ais_version = AisVersion {
            version_number: 1.31,
            version_code: AisCode::Production,
        };
        return new_ais_version
    }

    /// Fetches the manifest data.
    fn fetch_manifest() -> Result<serde_json::Value, UnifiedError> {
        let manifest_path = Self::fetch_manifest_path();
        match path_present(&manifest_path) {
            Ok(true) => {
                let mut file = File::open(&manifest_path)
                    .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;

                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;

                serde_json::from_slice(&buffer)
                    .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))
            }
            _ => {
                let generic_ais = AisInfo {
                    pages_id: None,
                    client_id: None,
                    machine_id: None,
                    machine_mac: Self::fetch_machine_mac(),
                    machine_ip: Self::fetch_machine_ip(),
                    ssh_events: 0,
                    system_version: AisVersion {
                        version_number: 0.00,
                        version_code: AisCode::Alpha,
                    },
                };

                serde_json::to_value(&generic_ais)
                    .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))
            }
        }
    }

    /// Fetches the manifest file path.
    fn fetch_manifest_path() -> PathType {
        PathType::Str("/etc/artisan.manifest".into())
    }

    /// Creates the manifest file.
    pub fn create_manifest(&self) -> Result<(), UnifiedError> {
        let json_data = serde_json::to_string(self)
            .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;

        let mut file = File::create(Self::fetch_manifest_path())
            .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;
        file.write_all(json_data.as_bytes())
            .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;

        Ok(())
    }

    /// Fetches the machine's MAC address.
    fn fetch_machine_mac() -> Option<String> {
        get_mac_address().ok().flatten().map(|mac| mac.to_string())
    }

    /// Fetches the machine's IP address.
    pub fn fetch_machine_ip() -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ais_info() {
        // Test creating a new AisInfo instance
        let ais_info = AisInfo::new().unwrap();

        // Assert that the fields are initialized correctly
        assert_eq!(ais_info.pages_id, None);
        assert_eq!(ais_info.client_id, None);
        assert!(ais_info.machine_mac.is_some());
        assert!(ais_info.machine_ip.is_some());
        assert_eq!(ais_info.ssh_events, 0);
        assert_eq!(ais_info.system_version.version_number, 1.31);
        assert_eq!(ais_info.system_version.version_code, AisCode::Production);
    }

    #[test]
    fn test_print_all() {
        // Test printing all information
        let ais_info = AisInfo {
            pages_id: Some("123".to_string()),
            client_id: Some("456".to_string()),
            machine_id: Some("789".to_string()),
            machine_mac: Some("00:11:22:33:44:55".to_string()),
            machine_ip: Some("192.168.1.100".to_string()),
            ssh_events: 5,
            system_version: AisVersion {
                version_number: 1.31,
                version_code: AisCode::ProductionCandidate,
            },
        };

        // Since print_all function prints to stdout, we'll just call it to check for errors
        ais_info.print_all();
    }

    #[test]
    fn test_fetch_manifest_path() {
        // Test fetching the manifest path
        let path = AisInfo::fetch_manifest_path();

        // Assert that the path is correct
        assert_eq!(path, PathType::Str("/etc/artisan.manifest".into()));
    }

    #[test]
    fn test_fetch_machine_mac() {
        // Test fetching the machine's MAC address
        let mac = AisInfo::fetch_machine_mac();

        // Assert that MAC address is not None
        assert!(mac.is_some());
    }

    #[test]
    fn test_fetch_machine_ip() {
        // Test fetching the machine's IP address
        let ip = AisInfo::fetch_machine_ip();

        // Assert that IP address is not None
        assert!(ip.is_some());
    }
}
