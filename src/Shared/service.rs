use crate::errors::{AisError, UnifiedError};
use chrono::{DateTime, Utc};
use std::fmt;
use systemctl::{self, Unit};

/// Enum representing different services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Services {
    PhpProcessor,
    WEBSERVER,
    SSHSERVER,
    MONITOR,
    FIREWALL,
    LOCKER,
}

/// Enum representing the status of a service.
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Running,
    Stopped,
    Error,
}

/// Enum representing memory information.
#[derive(Debug, Clone, PartialEq)]
pub enum Memory {
    MemoryConsumed(String),
}

/// Enum representing subprocesses information.
#[derive(Debug, Clone, PartialEq)]
pub enum SubProcesses {
    Pid(u64),
    Tasks(u64),
}

/// Struct representing information about a process.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub service: String,
    pub refered: Services,
    pub status: Status,
    pub memory: Memory,
    pub children: SubProcesses,
    pub timestamp: String,
}

/// Enum representing different types of processes.
#[derive(Debug, Clone)]
pub enum Processes {
    Services(Vec<ProcessInfo>),
}

impl Processes {
    /// Creates a new Processes instance containing information about various services.
    pub fn new() -> Result<Self, UnifiedError> {
        let mut data: Vec<ProcessInfo> = Vec::new();
        data.push(ProcessInfo::get_info(Services::WEBSERVER)?);
        data.push(ProcessInfo::get_info(Services::PhpProcessor)?);
        data.push(ProcessInfo::get_info(Services::FIREWALL)?);
        data.push(ProcessInfo::get_info(Services::MONITOR)?);
        data.push(ProcessInfo::get_info(Services::SSHSERVER)?);
        data.push(ProcessInfo::get_info(Services::LOCKER)?);

        Ok(Self::Services(data))
    }

    /// Updates the information of a specific service.
    pub fn update(service: Services) -> Result<ProcessInfo, UnifiedError> {
        ProcessInfo::get_info(service)
    }

    /// Iterates over the Processes enum and returns a vector of ProcessInfo.
    pub fn itr(&self) -> Vec<ProcessInfo> {
        match self {
            Processes::Services(data) => data.clone(),
        }
    }
}

impl Services {
    /// Retrieves information about the service.
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

    /// Restarts the service and returns a bool based on the running status after the restart.
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
    /// Retrieves information about a specific service.
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

/// Generates a timestamp string in the format: YYYY-MM-DD HH:MM:SS.
pub fn timestamp() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_services_display() {
        assert_eq!(format!("{}", Services::PhpProcessor), "php7.4-fpm.service");
        assert_eq!(format!("{}", Services::WEBSERVER), "apache2.service");
        assert_eq!(format!("{}", Services::SSHSERVER), "sshd.service");
        assert_eq!(format!("{}", Services::MONITOR), "netdata.service");
        assert_eq!(format!("{}", Services::FIREWALL), "ufw.service");
        assert_eq!(format!("{}", Services::LOCKER), "dusad.service");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", Status::Running), "active");
        assert_eq!(format!("{}", Status::Stopped), "stopped");
        assert_eq!(format!("{}", Status::Error), "Error occurred while checking");
    }

    #[test]
    fn test_memory_display() {
        assert_eq!(format!("{}", Memory::MemoryConsumed("2GB".to_string())), "2GB");
    }

    #[test]
    fn test_subprocesses_display() {
        assert_eq!(format!("{}", SubProcesses::Pid(123)), "123");
        assert_eq!(format!("{}", SubProcesses::Tasks(456)), "456");
    }

    #[test]
    fn test_timestamp() {
        let timestamp = timestamp();
        assert!(timestamp.len() > 0);
    }

}
