// THIS IS POTENTIALLY A BIG BOTTLENECK

use nix::unistd::{chown, Gid, Uid};
use service::{ProcessInfo, Processes, Status};
use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use system::{
    create_hash,
    errors::{SystemError, SystemErrorType},
    path_present, truncate, ClonePath, PathType,
};
use users::{Groups, Users, UsersCache};

use crate::{
    errors::{AisError, UnifiedError},
    service,
};

#[derive(Debug, Clone)]
pub struct Dusa {
    pub initialized: bool,
    pub service_name: String,
    pub debugging: bool,
    pub socket_path: PathType,
    pub process_status: Status,
}

pub enum Commands {
    EncryptFile(PathBuf, String, String), // path  owner name
    DecryptFile(String, String),          //Owner name
    DecryptText(String),                  // Cipher data
    EncryptText(String),                  // plain jane
    RemoveFile(String, String),           // owner name
}

impl Dusa {
    pub fn initialize(process_info: Arc<RwLock<Processes>>) -> Result<Self, UnifiedError> {
        let system_process_info = process_info
            .read()
            .map_err(|e| UnifiedError::from_ais_error(AisError::new(&e.to_string())))?;
        let dusa_process_info = system_process_info.itr();
        let dusa_data: &ProcessInfo = match dusa_process_info.get(5) {
            Some(d) => d,
            None => {
                return Err(UnifiedError::from_ais_error(AisError::new(
                    "Dusad system status unknown",
                )))
            }
        };
        let service_name: String = dusa_data.service.clone();
        let socket_path: PathType = PathType::Str("/var/run/dusa/dusa.sock".into());
        let debugging = true;
        let process_status: Status = dusa_data.status.clone();

        match &process_status {
            Status::Error => {
                return Err(AisError::EncryptionNotReady(Some(format!(
                    "Service: {} is not running or is in an unknown state",
                    &service_name
                )))
                .into());
            }
            _ => (),
        };

        if !path_present(&socket_path.clone_path())? {
            return Err(AisError::EncryptionNotReady(Some(format!(
                "Socket path {} is missing",
                &socket_path.display()
            )))
            .into());
        }

        Ok(Self {
            initialized: true,
            service_name,
            debugging,
            socket_path,
            process_status,
        })
    }
}

impl Commands {
    pub fn execute(&self) -> Result<Option<String>, UnifiedError> {
        match self {
            Commands::EncryptFile(path, owner, name) => {
                let retro_fit_path = PathType::PathBuf(path.to_path_buf());
                if !path_present(&retro_fit_path.clone_path())? {
                    return Err(SystemError::new(
                        system::errors::SystemErrorType::ErrorOpeningFile,
                    )
                    .into());
                }
                let (uid, gid) = Self::get_id();
                Self::set_file_ownership(path, uid, gid);

                let mut command_data: Vec<String> = vec![];
                command_data.push(String::from("insert"));
                command_data.push(owner.to_owned());
                command_data.push(name.to_owned());
                command_data.push(path.clone().into_os_string().into_string().unwrap());

                let message: String = Self::create_message(command_data);

                let response = Self::send_message(message)?;
                Ok(Some(response))
            }
            Commands::DecryptFile(_, _) => Ok(None),
            Commands::DecryptText(cipher_data) => {
                let mut command_data: Vec<String> = vec![];
                command_data.push(String::from("decrypt"));
                command_data.push(cipher_data.to_owned());

                let message: String = Self::create_message(command_data);

                let response: String = Self::send_message(message)?;
                Ok(Some(response))
            }
            Commands::EncryptText(data) => {
                let mut command_data: Vec<String> = vec![];
                command_data.push(String::from("encrypt"));
                command_data.push(data.to_owned());

                let message: String = Self::create_message(command_data);
                // notice(&message);

                let response = Self::send_message(message)?;
                Ok(Some(response))
            }
            Commands::RemoveFile(_, _) => Ok(None),
        }
    }

    fn create_message(mut data: Vec<String>) -> String {
        let current_uid: u32 = 0; // ais has to run as the root user
        data.push(format!("{}", current_uid));

        let command_string: String = data.join("*");
        let hexed_command: String = hex::encode(command_string);
        let hexed_hash: String =
            hex::encode(truncate(&create_hash(hexed_command.clone())[7..], 50));

        let mut secure_command_array: Vec<String> = vec![];
        secure_command_array.push(hexed_command);
        secure_command_array.push(hexed_hash);

        secure_command_array.join("Z")
    }

    fn send_message(command: String) -> Result<String, UnifiedError> {
        let socket_path: &Path = Path::new("/var/run/dusa/dusa.sock");

        let mut stream = UnixStream::connect(socket_path).map_err(|e| {
            SystemError::new_details(SystemErrorType::ErrorOpeningFile, &e.to_string())
        })?;

        stream.write_all(command.as_bytes()).map_err(|e| {
            SystemError::new_details(SystemErrorType::ErrorOpeningFile, &e.to_string())
        })?;
        stream.flush().map_err(|e| {
            SystemError::new_details(SystemErrorType::ErrorOpeningFile, &e.to_string())
        })?;

        let mut buffer = vec![0; 89200];
        let bytes_read = stream.read(&mut buffer).map_err(|e| {
            SystemError::new_details(SystemErrorType::ErrorOpeningFile, &e.to_string())
        })?;
        let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

        Ok(response)
    }

    fn get_id() -> (Uid, Gid) {
        let user_cache: UsersCache = UsersCache::new();
        let dusa_uid = user_cache.get_user_by_name("dusa").unwrap().uid();
        let dusa_gid = user_cache.get_group_by_name("dusa").unwrap().gid();

        (Uid::from_raw(dusa_uid), Gid::from_raw(dusa_gid))
    }

    fn set_file_ownership(path: &PathBuf, uid: Uid, gid: Gid) {
        chown(path, Some(uid), Some(gid)).expect("Failed to set file ownership");
    }
}
