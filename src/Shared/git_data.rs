use crate::errors::UnifiedError;
use crate::{encrypt::Commands, errors::AisError};
use pretty::warn;
use recs::errors::{RecsError, RecsErrorType};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
};
use system::{
    errors::{SystemError, SystemErrorType},
    path_present, PathType,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct GitAuth {
    pub user: String,
    pub repo: String,
    pub branch: String,
    pub token: String,
}

impl GitAuth {
    pub fn new() -> Result<Self, UnifiedError> {
        let file_location: &PathType = &PathType::Str("/etc/artisan.cf".into());
        let encrypted_credentials = match path_present(file_location) {
            Ok(true) => {
                let mut file = File::open(&file_location).map_err(|e| {
                    UnifiedError::from_system_error(SystemError::new_details(
                        SystemErrorType::ErrorOpeningFile,
                        &e.to_string(),
                    ))
                })?;
                let mut file_contents = String::new();
                file.read_to_string(&mut file_contents).map_err(|e| {
                    UnifiedError::from_system_error(SystemError::new_details(
                        SystemErrorType::ErrorReadingFile,
                        &e.to_string(),
                    ))
                })?;
                file_contents.replace("\n", "")
            }
            Ok(false) => {
                return Err(UnifiedError::from_system_error(SystemError::new_details(
                    SystemErrorType::ErrorOpeningFile,
                    "artisan credential file not found",
                )))
            }
            Err(e) => return Err(UnifiedError::from_system_error(e)),
        };

        let decrypt_command = Commands::DecryptText(encrypted_credentials);
        let decrypted_results = match decrypt_command.execute()? {
            Some(d) => d.replace("\0", ""),
            None => {
                return Err(UnifiedError::from_recs_error(RecsError::new_details(
                    RecsErrorType::Error,
                    "No data returned",
                )))
            }
        };

        let decrypted_bytes = hex::decode(decrypted_results).map_err(|e| {
            UnifiedError::from_system_error(SystemError::new_details(
                SystemErrorType::ErrorCreatingFile,
                &e.to_string(),
            ))
        })?;
        let decrypted_string = String::from_utf8(decrypted_bytes).map_err(|e| {
            UnifiedError::from_system_error(SystemError::new_details(
                SystemErrorType::ErrorCreatingFile,
                &e.to_string(),
            ))
        })?;
        let data: GitAuth = serde_json::from_str(&decrypted_string).map_err(|e| {
            UnifiedError::from_system_error(SystemError::new_details(
                SystemErrorType::ErrorCreatingFile,
                &e.to_string(),
            ))
        })?;

        Ok(data)
    }

    pub fn save(&self, file_path: &str) -> Result<(), UnifiedError> {
        // Serialize GitAuth to JSON
        let json_data = match serde_json::to_string(self) {
            Ok(d) => d,
            Err(e) => {
                return Err(UnifiedError::from_system_error(SystemError::new_details(
                    SystemErrorType::ErrorCreatingFile,
                    &e.to_string(),
                )))
            }
        };

        // Encrypt the JSON data
        let encrypt_command = Commands::EncryptText(json_data);
        let encrypted_data = match encrypt_command.execute()? {
            Some(data) => {
                warn(&data);
                data
            }
            None => {
                return Err(UnifiedError::from_system_error(SystemError::new(
                    SystemErrorType::ErrorCreatingFile,
                )))
            }
        };

        // Write the encrypted data to the file
        let mut file = match File::create(file_path) {
            Ok(d) => d,
            Err(e) => {
                return Err(UnifiedError::from_system_error(SystemError::new_details(
                    SystemErrorType::ErrorCreatingFile,
                    &e.to_string(),
                )))
            }
        };

        match file.write_all(encrypted_data.as_bytes()) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(UnifiedError::from_ais_error(AisError::new(&e.to_string()))),
        }
    }

    pub fn new_mock(user: &str, repo: &str) -> Self {
        Self {
            user: user.to_owned(),
            repo: repo.to_owned(),
            branch: "mull".to_ascii_uppercase(),
            token: "none".to_owned(),
        }
    }
}
