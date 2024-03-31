use pretty::{output, warn};
use shared::{
    emails::{Email, EmailSecure},
    errors::{AisError, UnifiedError},
    service::{Memory, Processes, Status},
    ais_data::AisInfo,
    git_data::GitAuth,
};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use sysinfo::{ProcessExt, System, SystemExt};
use system_shutdown::reboot;
use system::ClonePath;
use crate::{
    git_actions::GitAction,
    site_info::{SiteInfo, Updates},
    ssh_monitor::SshMonitor,
};

/// Updates the website continuously.
pub fn website_update_loop(
    system_data: Arc<RwLock<SiteInfo>>,
    ais_data: Arc<RwLock<AisInfo>>,
    git_creds: Arc<RwLock<GitAuth>>,
) -> Result<(), UnifiedError> {
    let mut site_data = acquire_write_lock(&system_data)?;

    let ais_info = acquire_read_lock(&ais_data)?;
    let git_info = acquire_read_lock(&git_creds)?;

    let new_site_data = SiteInfo::new(Arc::clone(&git_creds))?;

    match new_site_data.application_status {
        Updates::UpToDate => {
            site_data.application_status = new_site_data.application_status;
            drop(ais_info);
            Ok(())
        }
        Updates::OutOfDate => {
            let site_update_action = GitAction::Pull(site_data.application_folder.clone_path());
            let result: Result<bool, UnifiedError> = site_update_action.execute();
            match result {
                Ok(ok) => {
                    match ok {
                        true => {
                            site_data.application_status = new_site_data.application_status;

                            let mail = Email {
                                subject: "Applied Update".to_owned(),
                                body: format!("The system: {} Has just applied a new update from the repo: {}.", ais_info.machine_id.clone().unwrap_or_else(|| String::from("Failed to parse")), git_info.repo),
                            };
                            let phone_home = EmailSecure::new(mail)?;
                            phone_home.send()?;
                            drop(ais_info);
                            output("GREEN", "UPDATE FINISHED SUCCESSFULLY");
                            Ok(())
                        }
                        false => {
                            let mail = Email {
                                subject: "Update failed".to_owned(),
                                body: format!("The system: {} Has encountered and error applying an update from the repo: {}.", ais_info.machine_id.clone().unwrap_or_else(|| String::from("Failed to parse")), git_info.repo),
                            };
                            let phone_home = EmailSecure::new(mail)?;
                            phone_home.send()?;
                            drop(ais_info);
                            warn("An error occurred while updating");
                            Ok(())
                        }
                    }
                }
                Err(e) => Err(e),
            }
        }
    }
}

/// Updates machine-specific information.
pub fn machine_update_loop(ais_data: Arc<RwLock<AisInfo>>) -> Result<(), UnifiedError> {
    let ais_new_data = AisInfo::new()?;
    let mut ais_write_safe_data = acquire_write_lock(&ais_data)?;

    ais_write_safe_data.client_id = ais_new_data.client_id;
    ais_write_safe_data.machine_id = ais_new_data.machine_id;

    if ais_write_safe_data.machine_ip != ais_new_data.machine_ip {
        let mail = Email {
            subject: "Error Occurred".to_owned(),
            body: format!(
                "The system: {} Has encountered and error. The assigned IP address is not respected",
                ais_write_safe_data.machine_id.clone().unwrap_or_else(|| String::from("Failed to parse"))
            ),
        };
        let phone_home = EmailSecure::new(mail)?;
        phone_home.send()?;
        warn("An error occurred, Administrator notified");
    };
    if ais_write_safe_data.machine_mac != ais_new_data.machine_mac {
        let mail = Email {
            subject: "SOMETHING IS REALLY WRONG".to_owned(),
            body: format!("The system: {} Has encountered a major error. The MAC address on file is not the MAC address the system is reporting. The system is going offline.",
                          ais_write_safe_data.machine_id.clone().unwrap_or_else(|| String::from("Failed to parse"))),
        };
        let phone_home = EmailSecure::new(mail)?;
        phone_home.send()?;
        reboot().unwrap(); //todo  maybe handle this better one day
    };

    Ok(())
}

/// Updates system services and monitors their status.
pub fn service_update_loop(
    system_service_data: Arc<RwLock<Processes>>,
    ais_data: Arc<RwLock<AisInfo>>,
) -> Result<(), UnifiedError> {
    let service_data = acquire_read_lock(&system_service_data)?;
    let ais_info = acquire_read_lock(&ais_data)?;

    let mut data = Vec::new();

    for service_info in service_data.itr() {
        let new_service_info = service_info.refered.get_info()?;
        let new_service_to_update = new_service_info.clone();

        if service_info.status != new_service_info.status {
            match new_service_info.status {
                Status::Stopped => {
                    let email = Email {
                        subject: format!(
                            "{}: Service stopped",
                            ais_info.machine_id
                                .clone()
                                .unwrap_or_else(|| String::from("Failure parsing"))
                        ),
                        body: format!("The service {} stopped unexpectedly", service_info.service),
                    };
                    let phone_home = EmailSecure::new(email)?;
                    phone_home.send()?;
                    warn(&format!("Service {} has stopped. Emails has been sent", service_info.service));
                }
                Status::Error => {
                    let email = Email {
                        subject: format!(
                            "{}: Service in an unknown state",
                            ais_info.machine_id
                                .clone()
                                .unwrap_or_else(|| String::from("Failure parsing"))
                        ),
                        body: format!("The service {} stopped unexpectedly, attempting the restart automatically.", service_info.service),
                    };
                    let phone_home = EmailSecure::new(email)?;
                    match service_info.refered.restart()? {
                        true => {
                            warn(&format!(
                                "Service {} restarted successfully",
                                service_info.service
                            ));
                            drop(phone_home);
                        }
                        false => {
                            warn(&format!("Service {} has entered an erroneous state. Emails have been sent", service_info.service));
                            phone_home.send()?
                        }
                    }
                }
                Status::Running => {
                    let mail = Email {
                        subject: format!("{}: Service running", ais_info.machine_id.clone().unwrap_or_else(|| String::from("Failure parsing"))),
                        body: format!("The system: {} Is happy to report that the service: {} has entered the state {}.", ais_info.machine_id.clone()
                            .unwrap_or_else(|| String::from("Failure parsing")), new_service_info.service, new_service_info.status),
                    };
                    let phone_home = EmailSecure::new(mail)?;
                    phone_home.send()?;
                    output("GREEN", "Service started !");
                }
            }
        }

        match new_service_info.memory {
            Memory::MemoryConsumed(d) => {
                if d.contains("G") && d.contains("2.") {
                    let mail = Email {
                        subject: "Warning".to_owned(),
                        body: format!("The system: {} Wants you to know that: {} is consuming over 2G of resources. This should be safe to ignore.", ais_info.machine_id.clone()
                            .unwrap_or_else(|| String::from("Failure parsing")), new_service_info.service),
                    };
                    let phone_home = EmailSecure::new(mail)?;
                    phone_home.send()?;
                }
            }
        }
        data.push(new_service_to_update);
    }

    drop(service_data);

    let mut service_data_old = acquire_write_lock(&system_service_data)?;

    drop(ais_info);

    *service_data_old = Processes::Services(data);
    Ok(())
}

/// Monitors SSH connections.
pub fn monitor_ssh_connections(
    ssh_monitor: SshMonitor,
    ais_info: Arc<RwLock<AisInfo>>,
) -> Result<(), UnifiedError> {
    let mut system = System::new_all();
    system.refresh_all();

    for (_, process) in system.processes() {
        if process.name().contains("sshd") {
            return SshMonitor::process_ssh_connection(ssh_monitor, &process, ais_info);
        }
    }

    Ok(())
}

/// Helper function to acquire a read lock safely.
fn acquire_read_lock<T>(lock: &Arc<RwLock<T>>) -> Result<RwLockReadGuard<'_, T>, UnifiedError> {
    lock.try_read().map_err(|e| UnifiedError::from_ais_error(AisError::ThreadedDataError(
        Some(format!("Error acquiring read lock: {}", e.to_string())),
    )))
}

/// Helper function to acquire a write lock safely.
fn acquire_write_lock<T>(lock: &Arc<RwLock<T>>) -> Result<RwLockWriteGuard<'_, T>, UnifiedError> {
    lock.try_write().map_err(|e| UnifiedError::from_ais_error(AisError::ThreadedDataError(
        Some(format!("Error acquiring write lock: {}", e.to_string())),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_website_update_loop_up_to_date() {
        // Arrange
        let ais_data = Arc::new(RwLock::new(AisInfo::new().unwrap()));
        let git_creds = Arc::new(RwLock::new(GitAuth::new().unwrap()));
        let system_data = Arc::new(RwLock::new(SiteInfo::new(Arc::clone(&git_creds)).unwrap()));


        // Act
        let result = website_update_loop(system_data, ais_data, git_creds);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_machine_update_loop_success() {
        // Arrange
        let ais_data = Arc::new(RwLock::new(AisInfo::new().unwrap()));

        // Act
        let result = machine_update_loop(ais_data);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_service_update_loop_success() {
        // Arrange
        let system_service_data = Arc::new(RwLock::new(Processes::new().unwrap()));
        let ais_data = Arc::new(RwLock::new(AisInfo::new().unwrap()));

        // Act
        let result = service_update_loop(system_service_data, ais_data);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_monitor_ssh_connections_success() {
        // Arrange
        let ssh_monitor = SshMonitor::new();
        let ais_info = Arc::new(RwLock::new(AisInfo::new().unwrap()));

        // Act
        let result = monitor_ssh_connections(ssh_monitor, ais_info);

        // Assert
        assert!(result.is_ok());
    }
}