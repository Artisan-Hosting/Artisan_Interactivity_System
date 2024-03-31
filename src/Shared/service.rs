use crate::errors::{AisError, UnifiedError};
use chrono::{DateTime, Utc};
use std::fmt;
use systemctl::{self, Unit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Services {
    PhpProcessor,
    WEBSERVER,
    SSHSERVER,
    MONITOR,
    FIREWALL,
    LOCKER,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Running,
    Stopped,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Memory {
    MemoryConsumed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubProcesses {
    Pid(u64),
    Tasks(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub service: String,
    pub refered: Services,
    pub status: Status,
    pub memory: Memory,
    pub children: SubProcesses,
    pub timestamp: String,
}

#[derive(Debug, Clone)]
pub enum Processes {
    Services(Vec<ProcessInfo>),
}

impl Processes {
    pub fn new() -> Result<Self, UnifiedError> {
        let mut data: Vec<ProcessInfo> = Vec::new();
        // let webserver: ProcessInfo = ;
        data.push(ProcessInfo::get_info(Services::WEBSERVER)?);
        data.push(ProcessInfo::get_info(Services::PhpProcessor)?);
        data.push(ProcessInfo::get_info(Services::FIREWALL)?);
        data.push(ProcessInfo::get_info(Services::MONITOR)?);
        data.push(ProcessInfo::get_info(Services::SSHSERVER)?);
        data.push(ProcessInfo::get_info(Services::LOCKER)?);

        Ok(Self::Services(data))
    }

    pub fn update(service: Services) -> Result<ProcessInfo, UnifiedError> {
        ProcessInfo::get_info(service)
    }

    pub fn itr(&self) -> Vec<ProcessInfo> {
        match self {
            Processes::Services(data) => data.clone(),
        }
    }
}

impl Services {
    pub fn get_info(&self) -> Result<ProcessInfo, UnifiedError> {
        let unit_name: String = format!("{}", self.clone());
        let unit: Unit = match systemctl::Unit::from_systemctl(&unit_name) {
            Ok(d) => d,
            Err(e) => {
                return Err(UnifiedError::from_ais_error(AisError::SystemError(Some(
                    e.to_string(),
                ))));
            }
        };

        let status_data: Result<bool, std::io::Error> = unit.is_active();
        let status: Status = match status_data {
            Ok(true) => Status::Running,
            Ok(false) => Status::Stopped,
            Err(_) => Status::Error,
        };

        let memory_data: Option<String> = unit.memory;
        let memory: Memory = match memory_data {
            Some(d) => Memory::MemoryConsumed(d),
            None => Memory::MemoryConsumed(format!("{}B", 0.00.to_string())),
        };

        let (tasks, pid) = (unit.tasks, unit.pid);
        let children: SubProcesses = match (tasks, pid) {
            (Some(t), Some(_p)) => SubProcesses::Tasks(t),
            (_, _) => SubProcesses::Pid(0),
        };

        Ok(ProcessInfo {
            service: unit_name,
            status,
            memory,
            children,
            timestamp: timestamp(),
            refered: self.clone(),
        })
    }

    /// This function restarts the requested service. It returns a bool based on the running status after the restart
    pub fn restart(&self) -> Result<bool, UnifiedError> {
        let unit_name: String = format!("{}", self.clone());
        return match systemctl::restart(&unit_name) {
            Ok(_) => match systemctl::is_active(&unit_name) {
                Ok(d) => Ok(d),
                Err(e) => Err(UnifiedError::from_ais_error(AisError::new(&e.to_string()))),
            },
            Err(e) => Err(UnifiedError::from_ais_error(AisError::new(&e.to_string()))),
        };
    }
}

impl ProcessInfo {
    pub fn get_info(service: Services) -> Result<Self, UnifiedError> {
        let unit_name: String = format!("{}", &service);
        let unit: Unit = match systemctl::Unit::from_systemctl(&unit_name) {
            Ok(d) => d,
            Err(e) => {
                return Err(UnifiedError::from_ais_error(AisError::SystemError(Some(
                    e.to_string(),
                ))));
            }
        };

        let status_data: Result<bool, std::io::Error> = unit.is_active();
        let status: Status = match status_data {
            Ok(true) => Status::Running,
            Ok(false) => Status::Stopped,
            Err(_) => Status::Error,
        };

        let memory_data: Option<String> = unit.memory;
        let memory: Memory = match memory_data {
            Some(d) => Memory::MemoryConsumed(d),
            None => Memory::MemoryConsumed(format!("{}B", 0.00.to_string())),
        };

        let (tasks, pid) = (unit.tasks, unit.pid);
        let children: SubProcesses = match (tasks, pid) {
            (Some(t), Some(_p)) => SubProcesses::Tasks(t),
            (_, _) => SubProcesses::Pid(0),
        };

        Ok(Self {
            service: unit_name,
            status,
            memory,
            children,
            timestamp: timestamp(),
            refered: service,
        })
    }
}

// Displays

impl fmt::Display for Services {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &str = match self {
            Services::PhpProcessor => "php7.4-fpm.service",
            Services::WEBSERVER => "apache2.service",
            Services::SSHSERVER => "sshd.service",
            Services::MONITOR => "netdata.service",
            Services::FIREWALL => "ufw.service",
            Services::LOCKER => "dusad.service",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status: &str = match self {
            Status::Running => "active",
            Status::Stopped => "stopped",
            Status::Error => "Error occurred while checking",
        };
        write!(f, "{}", status)
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Memory::MemoryConsumed(d) => write!(f, "{}", d),
        }
    }
}

impl fmt::Display for SubProcesses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubProcesses::Pid(p) => write!(f, "{}", p),
            SubProcesses::Tasks(t) => write!(f, "{}", t),
        }
    }
}

// Helper function

pub fn timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}
