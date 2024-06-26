use crate::ssh_monitor::SshMonitor;
use pretty::{dump, notice, output, warn};
use shared::{
    ais_data::AisInfo,
    emails::{Email, EmailSecure},
    errors::{AisError, Caller, ErrorInfo, UnifiedError},
    git_actions::GitAction,
    git_data::GitCredentials,
    service::{Memory, Processes, Status},
    site_info::{SiteInfo, Updates},
};
use std::{
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    thread,
};
use sysinfo::System;
use system::{/*chown_recursive,*/ path_present, ClonePath, PathType};
use system_shutdown::reboot;
use systemstat::Duration;

pub fn website_update_loop(
    ais_data: Arc<RwLock<AisInfo>>,
    git_creds: Arc<RwLock<GitCredentials>>,
) -> Result<(), UnifiedError> {
    let ais_info = acquire_read_lock(
        &ais_data,
        Caller::Function(true, Some("Website Update Loop, ais_info".to_owned())),
    )?;

    let git_info = acquire_read_lock(
        &git_creds,
        Caller::Function(true, Some("Website Update Loop, git_info".to_owned())),
    )?;

    for git_credential in &git_info.auths {
        let new_site_data = SiteInfo::new(git_credential)?;
        // Ensure the path thats in the manifest exists before we try to update

        match path_present(&new_site_data.application_folder) {
            Ok(b) => match b {
                true => (), // Beautiful we are already initialized
                false => {
                    // Clone the git repo properly
                    let repo_url: String = format!(
                        "https://github.com/{}/{}.git",
                        git_credential.user, git_credential.repo
                    );
                    let repo_path: PathType = new_site_data.application_folder.clone_path();

                    match (GitAction::Clone {
                        repo_url,
                        destination: repo_path,
                    })
                    .execute()
                    {
                        Ok(d) => match d {
                            true => notice("New repo added"),          // We've cloned the repo
                            false => dump("Error while cloning repo"), // Since I have no error we'll let this be caught later
                        },
                        Err(e) => return Err(e),
                    }
                }
            },
            Err(e) => {
                return Err(UnifiedError::SystemError(
                    ErrorInfo::with_severity(
                        Caller::Function(true, Some(String::from("Website update loop"))),
                        shared::errors::Severity::Warning,
                    ),
                    e,
                ))
            }
        }

        // Perform site updates based on new_site_data
        match new_site_data.application_status {
            Updates::UpToDate => {
                GitAction::Switch {
                    branch: git_credential.branch.clone(),
                    destination: new_site_data.application_folder.clone_path(),
                }
                .execute()?;
                // chown_recursive(new_site_data.application_folder, Some(33), Some(33))?;
            }
            Updates::OutOfDate => {
                // Handle out-of-date scenario
                let site_update_action = GitAction::Pull {
                    target_branch: git_credential.branch.clone(),
                    destination: new_site_data.application_folder.clone_path(),
                };
                match site_update_action.execute() {
                    Ok(ok) => {
                        if ok {
                            // Successful update
                            let mail = Email {
                                subject: "Applied Update".to_owned(),
                                body: format!("The system: {} has just applied a new update from the repo: {}.", ais_info.machine_id.clone().unwrap_or_else(|| String::from("Failed to parse")), git_credential.repo),
                            };
                            let phone_home = EmailSecure::new(mail)?;
                            phone_home.send()?;
                            output("GREEN", "UPDATE FINISHED SUCCESSFULLY");
                        } else {
                            // Update failed
                            let mail = Email {
                                subject: "Update failed".to_owned(),
                                body: format!("The system: {} has encountered an error applying an update from the repo: {}.", ais_info.machine_id.clone().unwrap_or_else(|| String::from("Failed to parse")), git_credential.repo),
                            };
                            let phone_home = EmailSecure::new(mail)?;
                            phone_home.send()?;
                            warn("An error occurred while updating");
                        }
                    }
                    Err(e) => return Err(e),
                }
                // chown_recursive(new_site_data.application_folder, Some(33), Some(33))?;
            }
        }
    }
    Ok(())
}

/// Updates machine-specific information.
pub fn machine_update_loop(ais_data: Arc<RwLock<AisInfo>>) -> Result<(), UnifiedError> {
    let ais_new_data = AisInfo::new()?;
    let mut ais_write_safe_data = acquire_write_lock(
        &ais_data,
        Caller::Function(true, Some("Machine Update Loop".to_owned())),
    )?;

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

    drop(ais_write_safe_data);
    thread::sleep(Duration::from_nanos(100));
    Ok(())
}

/// Updates system services and monitors their status.
pub fn service_update_loop(
    system_service_data: Arc<RwLock<Processes>>,
    ais_data: Arc<RwLock<AisInfo>>,
) -> Result<(), UnifiedError> {
    let service_data = acquire_read_lock(
        &system_service_data,
        Caller::Function(true, Some("Service Update Loop, service_data".to_owned())),
    )?;
    let ais_info = acquire_read_lock(
        &ais_data,
        Caller::Function(true, Some("Service Update Loop, ais_info".to_owned())),
    )?;

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
                            ais_info
                                .machine_id
                                .clone()
                                .unwrap_or_else(|| String::from("Failure parsing"))
                        ),
                        body: format!("The service {} stopped unexpectedly", service_info.service),
                    };
                    let phone_home = EmailSecure::new(email)?;
                    phone_home.send()?;
                    warn(&format!(
                        "Service {} has stopped. Emails has been sent",
                        service_info.service
                    ));
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
                            warn(&format!(
                                "Service {} has entered an erroneous state. Emails have been sent",
                                service_info.service
                            ));
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
    drop(ais_info);
    drop(service_data);

    let mut service_data_old = acquire_write_lock(
        &system_service_data,
        Caller::Function(
            true,
            Some("Service Update Loop, New service data".to_owned()),
        ),
    )?;

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
pub fn acquire_read_lock<T: 'static>(
    lock: &Arc<RwLock<T>>,
    caller: Caller,
) -> Result<RwLockReadGuard<'_, T>, UnifiedError> {
    lock.read().map_err(|_| {
        UnifiedError::AisError(
            ErrorInfo::new(caller),
            AisError::ThreadedDataError(Some(format!("Error acquiring Read lock"))),
        )
    })
}

/// Helper function to acquire a write lock safely.
pub fn acquire_write_lock<T: 'static>(
    lock: &Arc<RwLock<T>>,
    caller: Caller,
) -> Result<RwLockWriteGuard<'_, T>, UnifiedError> {
    lock.write().map_err(|_| {
        UnifiedError::AisError(
            ErrorInfo::new(caller),
            AisError::ThreadedDataError(Some(format!("Error acquiring Write lock"))),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_update_loop_success() {
        // Arrange
        let ais_data = Arc::new(RwLock::new(AisInfo::new().unwrap()));

        // Act
        let result = machine_update_loop(ais_data);

        // Assert
        assert!(result.is_ok());
    }

    #[cfg(feature = "software")]
    #[test]
    fn test_service_update_loop_success() {
        // Arrange
        let system_service_data = Arc::new(RwLock::new(Processes::new().unwrap()));
        let ais_data = Arc::new(RwLock::new(AisInfo::new().unwrap()));

        // Act
        let result = service_update_loop(system_service_data, ais_data);

        // Assert
        assert!(result.is_ok()); // TODO will fail on dev computers
    }

    // #[test] // TODO better setup this test or test its components
    // fn test_monitor_ssh_connections_success() {
    //     // Arrange
    //     let ssh_monitor = SshMonitor::new();
    //     let ais_info = Arc::new(RwLock::new(AisInfo::new().unwrap()));

    //     // Act
    //     let result = monitor_ssh_connections(ssh_monitor, ais_info);

    //     // Assert
    //     assert!(result.is_ok());
    // }
}
