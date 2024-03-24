use chrono::{Local, Duration};
use pretty::warn;
use serde::{Deserialize, Serialize};
use shared::{
    ais_data::AisInfo,
    emails::{Email, EmailSecure},
    errors::{AisError, UnifiedError},
};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};
use sysinfo::{Process, ProcessExt};

#[derive(Debug, Clone)]
pub enum SshMonitor {
    SeenProcesses(Arc<RwLock<HashSet<i32>>>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SshInfo {
    pub time_stamp: String,
    pub system_ip: String,
    pub remote_ip: String,
    pub system_user: String,
    pub connection_duration: String,
    pub protocol_version: String,
    pub authentication_status: String,
    pub geolocation: String,
    pub ssh_key_usage: String,
}

impl SshInfo {
    pub fn new(system_ip: String, remote_ip: String, system_user: String) -> Self {
        Self {
            time_stamp: Local::now().to_string(),
            system_ip,
            remote_ip,
            system_user,
            connection_duration: String::new(),
            protocol_version: String::new(),
            authentication_status: String::new(),
            geolocation: String::new(),
            ssh_key_usage: String::new(),
        }
    }

    pub fn prepare(&mut self) -> Email {
        let subject = "SSH ACCESS AUDIT".to_string();
        let body = format!(
            "SSH ACCESS NOTIFICATION\nAt {} THE HOST {} WAS ACCESSED \nBY {} FROM REMOTE IP: {}.",
            self.time_stamp, self.system_ip, self.system_user, self.remote_ip
        );

        Email { subject, body }
    }
}

impl SshMonitor {
    pub fn new() -> Self {
        Self::SeenProcesses(Arc::new(RwLock::new(HashSet::new())))
    }

    pub fn access(&self) -> Arc<RwLock<HashSet<i32>>> {
        match self {
            SshMonitor::SeenProcesses(d) => d.clone(),
        }
    }

    pub fn process_ssh_connection(
        mut self,
        process: &Process,
        ais_info: Arc<RwLock<AisInfo>>,
    ) -> Result<(), UnifiedError> {
        let process_id = process.pid();
        if let SshMonitor::SeenProcesses(seen_processes) = &mut self {
            let mut seen_processes = seen_processes.write().map_err(|e| UnifiedError::AisError(AisError::ThreadedDataError(Some(e.to_string()))))?;
            if seen_processes.insert(process_id) {
                let (auth, username, protocol_version) = self.validate_users(process.cmd().join(" "));
                if !auth {
                    warn("Failed SSH authentication attempt detected.");
                    return Ok(());
                }
                let ais_info = ais_info.read().map_err(|e| UnifiedError::AisError(AisError::ThreadedDataError(Some(e.to_string()))))?;
                let remote_ip = process.tcp_connections().get(0).map(|c| c.remote_address.to_string()).unwrap_or_else(|| "Unknown".to_string());
                let mut ssh_info = SshInfo::new(ais_info.machine_ip.clone().unwrap_or(String::from("Parsing Error")), remote_ip, username.clone().unwrap_or(String::from("Pasring Error")));
                // Calculate connection duration
                let connection_duration = Local::now().signed_duration_since(Local::now() - Duration::seconds(process.start_time() as i64));
                ssh_info.connection_duration = connection_duration.to_string();
                // Capture protocol version
                ssh_info.protocol_version = protocol_version.to_string();
                // Capture authentication status
                ssh_info.authentication_status = "Authenticated".to_string();
                // Perform geolocation lookup (assuming a function named `lookup_geolocation`)
                // ssh_info.geolocation = lookup_geolocation(&ssh_info.remote_ip);
                // Log SSH key usage
                // ssh_info.ssh_key_usage = determine_ssh_key_usage(process);
                // info!("{}", ssh_info.prepare());
                Ok(())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn create_ssh_report(
        ais_info: Arc<RwLock<AisInfo>>,
        username: String,
        failed_attempts: usize,
    ) -> Result<(), UnifiedError> {
        let mut ais_data = ais_info.write().map_err(|e| UnifiedError::AisError(AisError::ThreadedDataError(Some(e.to_string()))))?;

        let time_stamp = Local::now().to_string();
        let system_ip = ais_data.machine_ip.clone().ok_or_else(|| UnifiedError::AisError(AisError::new("The IP address provided was not valid")))?;
        let system_user = username;
        let mut ssh_report = SshInfo {
            time_stamp,
            system_ip,
            remote_ip: String::new(), // Placeholder value
            system_user,
            connection_duration: String::new(), // Placeholder value
            protocol_version: String::new(), // Placeholder value
            authentication_status: String::new(), // Placeholder value
            geolocation: String::new(), // Placeholder value
            ssh_key_usage: String::new(), // Placeholder value
        };
        let ssh_report_data = ssh_report.prepare();
        ais_data.ssh_events += 1;
        warn(&format!("SSH events: {}", ais_data.ssh_events));
        let secure_email: EmailSecure = EmailSecure::new(ssh_report_data)?;
        drop(ais_data);

        secure_email.send()
    }

    pub fn validate_users(&self, mut data: String) -> (bool, Option<String>, String) {
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
            "Protocol Version".to_string(), // Placeholder value
        )
    }
}
