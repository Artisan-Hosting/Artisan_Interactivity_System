//! # Main Module
//!
//! This module contains the main entry point of the application.

pub mod loops;
pub mod ssh_monitor;

use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use pretty::{notice, warn};
use shared::{
    ais_data::AisInfo,
    encrypt::Dusa,
    errors::{UnifiedError, UnifiedErrorResult},
    git_data::GitCredentials,
    service::Processes,
};

use loops::{
    machine_update_loop, monitor_ssh_connections, service_update_loop, website_update_loop
};
use ssh_monitor::SshMonitor;

/// Entry point of the application
fn main() {
    // Initializing GitHub information
    let git_creds_data: GitCredentials = GitCredentials::new().unwrap();
    warn(&format!("{:?}", &git_creds_data));
    let git_creds_rw: Arc<RwLock<GitCredentials>> = Arc::new(RwLock::new(git_creds_data));

    // Getting system service information
    let system_services_data: UnifiedErrorResult<Processes> =
        UnifiedErrorResult::new(Processes::new());
    let system_service_rw: Arc<RwLock<Processes>> =
        Arc::new(RwLock::new(system_services_data.unwrap()));

    // Initializing Dusa connection
    let dusa_data: UnifiedErrorResult<Dusa> =
        UnifiedErrorResult::new(Dusa::initialize(Arc::clone(&system_service_rw)));
    let _dusa_data_rw: Arc<RwLock<Dusa>> = Arc::new(RwLock::new(dusa_data.unwrap()));

    // Initialize the AIS information
    let ais_data: UnifiedErrorResult<AisInfo> = UnifiedErrorResult::new(AisInfo::new());
    let ais_rw: Arc<RwLock<AisInfo>> = Arc::new(RwLock::new(ais_data.unwrap()));

    // Initializing the SSH monitor
    let ssh_data: SshMonitor = SshMonitor::new();

    // Spawn a thread to log operational status periodically
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(600)); // Every 5 mins we just say hello
        notice("Operational");
    });

    // Main application loop
    loop {
        // Initialize handlers for various tasks
        let handlers = initialize_handlers(
            // system_data_rw.clone(),
            ais_rw.clone(),
            git_creds_rw.clone(),
            system_service_rw.clone(),
            ssh_data.clone(),
        );

        // Join all threads and handle errors
        for handler in handlers {
            match handler.join() {
                Ok(result) => match result {
                    Ok(_) => (),
                    Err(e) => warn(&format!("Thread failed with error: {:?}", e)),
                },
                Err(e) => println!("Thread panicked: {:?}", e),
            }
        }

        // Introduce a sleep to reduce CPU usage
        thread::sleep(Duration::from_nanos(90)); // Adjust the duration as needed
    }
}

/// Initialize handlers for various tasks
fn initialize_handlers(
    ais_rw: Arc<RwLock<AisInfo>>,
    git_creds_rw: Arc<RwLock<GitCredentials>>,
    system_service_rw: Arc<RwLock<Processes>>,
    ssh_data: SshMonitor,
) -> Vec<thread::JoinHandle<Result<(), UnifiedError>>> {
    // Spawn a thread to monitor SSH connections
    let monitor_ssh = {
        let ais_rw_clone = Arc::clone(&ais_rw);
        let ssh_data_clone = ssh_data.clone();
        thread::spawn(move || monitor_ssh_connections(ssh_data_clone, ais_rw_clone))
    };

    // Spawn a thread to monitor machine updates
    let machine_monitor = {
        let ais_rw_clone = Arc::clone(&ais_rw);
        thread::spawn(move || machine_update_loop(ais_rw_clone))
    };

    // Spawn a thread to monitor system services
    let service_monitor = {
        let system_service_rw_clone = Arc::clone(&system_service_rw);
        let ais_rw_clone = Arc::clone(&ais_rw);
        thread::spawn(move || service_update_loop(system_service_rw_clone, ais_rw_clone))
    };

    // Spawn a thread to monitor website updates
    let website_monitor = {
        let ais_rw_clone = Arc::clone(&ais_rw);
        let git_creds_rw_clone = Arc::clone(&git_creds_rw);
        thread::spawn(move || {
            website_update_loop(ais_rw_clone, git_creds_rw_clone)
        })
    };

    vec![
        monitor_ssh,
        machine_monitor,
        service_monitor,
        website_monitor,
    ]
}
