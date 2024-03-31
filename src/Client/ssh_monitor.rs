use chrono::Local;
use pretty::warn;
use shared::ais_data::AisInfo;
use shared::errors::{AisError, UnifiedError};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};
use sysinfo::{Process, ProcessExt};

use shared::emails::{Email, EmailSecure};

#[derive(Debug, Clone)]
pub enum SshMonitor {
    SeenProcesses(Arc<RwLock<HashSet<i32>>>),
}

pub struct SshInfo {
    pub time_stamp: String,
    pub system_ip: String,
    pub system_user: String,
    pub priority_status: bool,
    // pub origin_known: bool, implement ip tracking later
}

impl SshInfo {
    pub fn prepare(&mut self) -> Email {
        let importance = if self.priority_status {
            String::from("HIGH")
        } else {
            String::from("LOW")
        };

        let origin = String::from("UNKNOWN");

        let subject = format!("SSH ACCESS AUDIT {} IMPORTANCE", importance);
        let body = format!(
            "SSH ACCESS NOTIFICATION\nAt {} THE HOST {} WAS ACCESSED \nBY {}, FROM AN ORIGIN {}.",
            self.time_stamp, self.system_ip, self.system_user, origin
        );

        Email { subject, body }
    }
}

impl SshMonitor {
    pub fn new() -> Self {
        Self::SeenProcesses(Arc::new(RwLock::new(HashSet::new())))
    }

    pub fn access(self) -> Arc<RwLock<HashSet<i32>>> {
        match self {
            SshMonitor::SeenProcesses(d) => d.clone(),
        }
    }

    pub fn process_ssh_connection(
        self,
        process: &Process,
        ais_info: Arc<RwLock<AisInfo>>,
    ) -> Result<(), UnifiedError> {
        let binding = self.clone().access();
        let mut seen_processes = match binding.write() {
            Ok(d) => d,
            Err(e) => {
                return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                    Some(e.to_string()),
                )))
            }
        };

        if seen_processes.insert(process.pid()) {
            let (auth, username) = self.validate_users(process.cmd().join(" "));

            if auth && username.is_none() {
                return Err(UnifiedError::from_ais_error(AisError::SshUnknownUser(
                    Some(String::from("Unknown")),
                )));
            };

            match auth {
                true => {
                    return SshMonitor::create_ssh_report(
                        ais_info,
                        username.unwrap_or_else(|| "Already established connection?".to_string()),
                    );
                }
                false => {
                    warn("ssh connections are active");
                    return Ok(());
                    // return SshMonitor::create_ssh_report(
                    //     ais_info,
                    //     username.unwrap_or_else(|| "Already established connection?".to_string()),
                    // );
                }
            }
        } else {
            return Ok(());
        }
    }

    pub fn create_ssh_report(
        ais_info: Arc<RwLock<AisInfo>>,
        username: String,
    ) -> Result<(), UnifiedError> {
        let mut ais_data = match ais_info.write() {
            Ok(d) => d,
            Err(e) => {
                return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(
                    Some(String::from(&e.to_string())),
                )))
            }
        };

        let time_stamp = Local::now().to_string();
        let system_ip = &ais_data.machine_ip;
        let system_user = username;
        let priority_status = true;
        let mut ssh_report = SshInfo {
            time_stamp,
            system_ip: match system_ip {
                Some(d) => String::from(d.clone()),
                None => {
                    return Err(UnifiedError::from_ais_error(AisError::new(
                        "The ip address provided was not valid",
                    )))
                }
            },
            system_user,
            priority_status,
        };
        let ssh_report_data = ssh_report.prepare();
        ais_data.ssh_events += 1;
        warn(&format!("Ssh events: {}", ais_data.ssh_events));
        let secure_email: EmailSecure = EmailSecure::new(ssh_report_data)?;
        drop(ais_data);

        return secure_email.send();
    }

    pub fn validate_users(&self, mut data: String) -> (bool, Option<String>) {
        let user_list_critical = vec![
            "dwhitfield".to_string(),
            "root".to_string(),
            "system".to_string(),
            "web_admin".to_string(),
        ];

        if data.contains("[priv]") {
            data = "[auth event]".to_string()
        };
        if data.contains("[net]") {
            data = "[auth event]".to_string()
        };
        if data.contains("[listener]") {
            data = "[server start]".to_string()
        };

        let data = data.replace("sshd:", "");
        let data = data.replace(" ", "");
        let data_expanded = data.split('@');
        let data_parts: Vec<&str> = data_expanded.collect();

        let contains = user_list_critical.contains(&format!("{}", data_parts[0]));

        (
            contains,
            if contains {
                Some(format!("{}", data_parts[0]))
            } else {
                None
            },
        )
    }
}
