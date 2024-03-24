pub mod git_actions;
pub mod loops;
pub mod site_info;
pub mod ssh_monitor;

use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use loops::{
    machine_update_loop, monitor_ssh_connections, service_update_loop, website_update_loop,
};
use pretty::warn;
use shared::ais_data::AisInfo;
use shared::git_data::GitAuth;
use shared::{encrypt::Dusa, errors::UnifiedError, service::Processes};
use site_info::SiteInfo;
use ssh_monitor::SshMonitor;

fn main() {
    // Getting system service information
    let system_services_data: Processes = Processes::new().unwrap();
    let system_service_rw: Arc<RwLock<Processes>> = Arc::new(RwLock::new(system_services_data));

    // Initializing dusa connection
    let dusa_data: Dusa = Dusa::initialize(Arc::clone(&system_service_rw)).unwrap();
    let _dusa_data_rw: Arc<RwLock<Dusa>> = Arc::new(RwLock::new(dusa_data));

    // Initialize the ais information
    let ais_data: AisInfo = AisInfo::new();
    let ais_rw: Arc<RwLock<AisInfo>> = Arc::new(RwLock::new(ais_data));

    // Initializing git hub information
    let git_creds_data: GitAuth = GitAuth::new().unwrap();
    let git_creds_rw: Arc<RwLock<GitAuth>> = Arc::new(RwLock::new(git_creds_data));

    // Initializing site data information
    let system_data: SiteInfo =
        SiteInfo::new(Arc::clone(&git_creds_rw)).unwrap();
    let system_data_rw: Arc<RwLock<SiteInfo>> = Arc::new(RwLock::new(system_data));

    // Initializing the ssh monitor array
    let ssh_data: SshMonitor = SshMonitor::new();

    loop {
        // Get the handlers
        let handlers: Vec<thread::JoinHandle<Result<(), UnifiedError>>> = initialize_handlers(
            system_data_rw.clone(),
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

        // Introduce a sleep or break
        thread::sleep(Duration::from_nanos(500)); // Adjust the duration as needed
    }
}

fn initialize_handlers(
    system_data_rw: Arc<RwLock<SiteInfo>>,
    ais_rw: Arc<RwLock<AisInfo>>,
    git_creds_rw: Arc<RwLock<GitAuth>>,
    system_service_rw: Arc<RwLock<Processes>>,
    ssh_data: SshMonitor,
) -> Vec<thread::JoinHandle<Result<(), UnifiedError>>> {
    let monitor_ssh = {
        let _system_data_rw_clone = Arc::clone(&system_data_rw);
        let ais_rw_clone = Arc::clone(&ais_rw);
        let ssh_data_clone = ssh_data.clone(); // Clone the SshMonitor
        thread::spawn(move || monitor_ssh_connections(ssh_data_clone, ais_rw_clone))
    };

    let service_monitor = {
        let system_service_rw_clone = Arc::clone(&system_service_rw);
        let ais_rw_clone = Arc::clone(&ais_rw);
        thread::spawn(move || {
            service_update_loop(
                system_service_rw_clone,
                ais_rw_clone,
            )
        })
    };

    let website_monitor = {
        let system_data_rw_clone = Arc::clone(&system_data_rw);
        let ais_rw_clone = Arc::clone(&ais_rw);
        let git_creds_rw_clone = Arc::clone(&git_creds_rw);
        thread::spawn(move || {
            website_update_loop(system_data_rw_clone, ais_rw_clone, git_creds_rw_clone)
        })
    };

    let machine_monitor = {
        let ais_rw_clone = Arc::clone(&ais_rw);
        thread::spawn(move || {
            machine_update_loop(ais_rw_clone)
        })
    };

    vec![
        monitor_ssh,
        machine_monitor,
        service_monitor,
        website_monitor,
    ]
}
