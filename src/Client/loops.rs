use pretty::{output, warn};
use shared::emails::{Email, EmailSecure};
use shared::errors::{AisError, UnifiedError};
use shared::service::{Memory, ProcessInfo, Processes, Status};
use shared::{ais_data::AisInfo, git_data::GitAuth};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use sysinfo::{ProcessExt, System, SystemExt};
use system::ClonePath;
use system_shutdown::reboot;

use crate::{
    git_actions::GitAction,
    site_info::{SiteInfo, Updates},
    ssh_monitor::SshMonitor,
};

pub fn website_update_loop(
    system_data: Arc<RwLock<SiteInfo>>,
    ais_data: Arc<RwLock<AisInfo>>,
    git_creds: Arc<RwLock<GitAuth>>,
) -> Result<(), UnifiedError> {
    let mut site_data: std::sync::RwLockWriteGuard<'_, SiteInfo> = match system_data.try_write() {
        Ok(d) => d,
        Err(e) => {
            return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                Some(format!("website update {}", e.to_string())),
            )))
        }
    };

    let ais_info: std::sync::RwLockReadGuard<'_, AisInfo> = match ais_data.try_read() {
        Ok(d) => d,
        Err(e) => {
            return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                Some(format!("website update {}", e.to_string())),
            )))
        }
    };

    let git_info: std::sync::RwLockReadGuard<'_, GitAuth> = match git_creds.try_read() {
        Ok(d) => d,
        Err(e) => {
            return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                Some(format!("website update {}", e.to_string())),
            )))
        }
    };

    let new_site_data: SiteInfo = SiteInfo::new(Arc::clone(&git_creds))?;

    match new_site_data.application_status {
        Updates::UpToDate => {
            site_data.application_status = new_site_data.application_status;
            drop(ais_info);
            return Ok(());
        }
        Updates::OutOfDate => {
            let site_update_action: GitAction =
                GitAction::Pull(site_data.application_folder.clone_path());
            let result: Result<bool, UnifiedError> = site_update_action.execute();
            match result {
                Ok(ok) => {
                    match ok {
                        true => {
                            site_data.application_status = new_site_data.application_status;

                            let mail: Email = Email {
                                    subject: "Applied Update".to_owned(),
                                    body: format!("The system: {} Has just applied a new update from the repo: {}.", ais_info.machine_id.clone().unwrap_or(String::from("Failed to parse")), git_info.repo),
                                };
                            let phone_home: EmailSecure = EmailSecure::new(mail)?;
                            phone_home.send()?;
                            drop(ais_info);
                            output("GREEN", "UPDATE FINISHED SUCCESSFULLY");
                            return Ok(());
                        }
                        false => {
                            let mail: Email = Email {
                                    subject: "Update failed".to_owned(),
                                    body: format!("The system: {} Has encountered and error appling an update from the repo: {}.", ais_info.machine_id.clone().unwrap_or(String::from("Failed to parse")), git_info.repo),
                                };
                            let phone_home: EmailSecure = EmailSecure::new(mail)?;
                            phone_home.send()?;
                            drop(ais_info);
                            warn("An error occoured while updating");
                            return Ok(());
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}

pub fn machine_update_loop(ais_data: Arc<RwLock<AisInfo>>) -> Result<(), UnifiedError> {
    let ais_new_data: AisInfo = AisInfo::new()?;
    let mut ais_write_safe_data: RwLockWriteGuard<'_, AisInfo> = match ais_data.write() {
        Ok(d) => d,
        Err(e) => {
            return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                Some(format!("machine update {}", e.to_string())),
            )))
        }
    };

    ais_write_safe_data.client_id = ais_new_data.client_id;
    ais_write_safe_data.machine_id = ais_new_data.machine_id;

    if ais_write_safe_data.machine_ip != ais_new_data.machine_ip {
        let mail: Email =
            Email {
                subject: "Error Occoured".to_owned(),
                body: format!(
                "The system: {} Has encountered and error. The assigned ip addr is not respected",
                ais_write_safe_data.machine_id.clone().unwrap_or(String::from("Failed to parse"))
            ),
            };
        let phone_home: EmailSecure = EmailSecure::new(mail)?;
        phone_home.send()?;
        warn("An error occoured, Administrator notified");
        return Ok(());
    };
    if ais_write_safe_data.machine_mac != ais_new_data.machine_mac {
        let mail: Email = Email {
                subject: "SOMETHING IS REALLY WRONG".to_owned(),
                body: format!("The system: {} Has encountered a major error. The mac addr on file is not the mac addr the system is reporting. The system is going offline.", 
                    ais_write_safe_data.machine_id.clone().unwrap_or(String::from("Failed to parse"))),
            };
        let phone_home: EmailSecure = EmailSecure::new(mail)?;
        phone_home.send()?;
        reboot().unwrap(); //todo  maybe handle this better one day
    };

    drop(ais_write_safe_data);
    return Ok(());
}

#[allow(unused_assignments)]
pub fn service_update_loop(
    system_service_data: Arc<RwLock<Processes>>,
    ais_data: Arc<RwLock<AisInfo>>,
) -> Result<(), UnifiedError> {
    // system service monitor
    let service_data: RwLockReadGuard<'_, Processes> = match system_service_data.try_read() {
        Ok(d) => d,
        Err(e) => {
            return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                Some(format!("service update {}", e.to_string())),
            )))
        }
    };

    let ais_info: std::sync::RwLockReadGuard<'_, AisInfo> = match ais_data.try_read() {
        Ok(d) => d,
        Err(e) => {
            return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                Some(format!("service update {}", e.to_string())),
            )))
        }
    };

    let mut data: Vec<ProcessInfo> = Vec::new();

    // loop through all of the services in service data
    for service_info in service_data.itr() {
        let new_service_info: ProcessInfo = service_info.refered.get_info()?;
        let new_service_to_update: ProcessInfo = new_service_info.clone();

        // Running some check on the services to make sure they are in a functional state
        if service_info.status != new_service_info.status {
            match new_service_info.status {
                Status::Stopped => {
                    let email: Email = Email {
                        subject: format!(
                            "{}: Service stopped",
                            &ais_info
                                .machine_id
                                .clone()
                                .unwrap_or(String::from("Failiure parsing"))
                        ),
                        body: format!("The service {} stopped unexpectedly", service_info.service),
                    };
                    let phone_home: EmailSecure = EmailSecure::new(email)?;
                    phone_home.send()?;
                    warn(&format!("Service {} has stopped. Emails has been sent", service_info.service));
                    // Send an email that a service is stopped and
                }
                Status::Error => {
                    let email: Email = Email {
                        subject: format!(
                            "{}: Service in an unknown state",
                            &ais_info
                                .machine_id
                                .clone()
                                .unwrap_or(String::from("Failiure parsing"))
                        ),
                        body: format!("The service {} stopped unexpectedly, attempting the restart automatically.", service_info.service),
                    };
                    let phone_home: EmailSecure = EmailSecure::new(email)?;
                    match service_info.refered.restart()? {
                        true => {
                            warn(&format!(
                                "Service {} restarted sucessfully",
                                service_info.service
                            ));
                            drop(phone_home);
                        }
                        false => {
                            warn(&format!("Service {} has entered and erroneous state. Emails has been sent", service_info.service));
                            phone_home.send()?
                        }
                    }
                }
                Status::Running => {
                    let mail: Email = Email {
                            subject: format!("{}: Service running", &ais_info.machine_id.clone().unwrap_or(String::from("Failiure parsing"))),
                            body: format!("The system: {} Is happy to report that the service: {} has entered the state {}.", &ais_info.machine_id.clone()
                            .unwrap_or(String::from("Failiure parsing")), new_service_info.service, new_service_info.status),
                        };
                    let phone_home: EmailSecure = EmailSecure::new(mail)?;
                    phone_home.send()?;
                    output("GREEN", "Service started !");
                }
            }
        }

        match new_service_info.memory {
            Memory::MemoryConsumed(d) => {
                if d.contains("G") && d.contains("2.") {
                    let mail: Email = Email {
                            subject: "Warning".to_owned(),
                            body: format!("The system: {} Wants you to know that: {} is consuming over 2G of resources this should be safe to ignore.", &ais_info.machine_id.clone()
                            .unwrap_or(String::from("Failiure parsing")), new_service_info.service),
                        };
                    let phone_home: EmailSecure = EmailSecure::new(mail)?;
                    phone_home.send()?;
                }
            }
        }
        data.push(new_service_to_update);
    }

    drop(service_data);

    let mut service_data_old: RwLockWriteGuard<'_, Processes> =
        match system_service_data.try_write() {
            Ok(d) => d,
            Err(e) => {
                return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                    Some(format!("service update {}", e.to_string())),
                )))
            }
        };

    drop(ais_info);

    *service_data_old = Processes::Services(data);
    drop(service_data_old);

    return Ok(());
}

pub fn monitor_ssh_connections(
    ssh_monitor: SshMonitor,
    ais_info: Arc<RwLock<AisInfo>>,
) -> Result<(), UnifiedError> {
    let mut system: System = System::new_all();
    system.refresh_all();

    for (_, process) in system.processes() {
        if process.name().contains("sshd") {
            return SshMonitor::process_ssh_connection(ssh_monitor, &process, ais_info);
        } else {
            return Ok(());
        }
    }

    Ok(())
}
