use std::process::Command;
use hostname::set;
use pretty::{halt, notice, output};
use shared::errors::*;
use shared::service::Services;
use shared::{ais_data::AisInfo, service::ProcessInfo};
use system::{create_hash, make_file, path_present, truncate, PathType};

#[allow(dead_code)]
struct SystemPaths {
    service_location: PathType,
    timer_location: PathType,
    stub_location: PathType,
    root_location: PathType,
}

impl SystemPaths {
    fn new() -> Self {
        Self {
            root_location: PathType::Str("/opt/artisan/".into()),
            service_location: PathType::Str("/opt/artisan/services".into()),
            timer_location: PathType::Str("/opt/artisan/timers".into()),
            stub_location: PathType::Str("/opt/artisan/stubs".into()),
        }
    }
}

// * Defining the paths

fn main() {

    let _dirs: SystemPaths = SystemPaths::new();
    let installed_path = format!("/opt/artisan/{}", truncate(&create_hash(String::from("Initialized")), 7));

    let system_clean: bool = match path_present(&PathType::Content(installed_path.clone())) {
        Ok(b) => b,
        Err(err) => {
            halt(&format!("{:#?}", err.details));
            panic!()
        }
    };

    // Performing the proper initalizations once the manifest is deleted.
    match system_clean {
        true => output("GREEN", "System Already Initialized"), // The manifest does not exist somethings wonky
        false => {
 
            match path_present(&PathType::Str("/etc/systemd/system/ais.service".into())) {
                Ok(b) => match b {
                    true => notice("Service files present"),
                    false => halt("Service files not present"),
                },
                Err(e) => halt(&format!("{}", e)),
            }

            // ! INITIALIZING SSHD
            let ssh_process: UnifiedErrorResult<ProcessInfo> =
                UnifiedErrorResult::new(Services::SSHSERVER.get_info());

            let ssh_unit = match systemctl::Unit::from_systemctl(&ssh_process.unwrap().service) {
                Ok(d) => d,
                Err(err) => {
                    halt(&format!("{}", &err.to_string()));
                    panic!();
                }
            };

            // verifing we stoped ssh
            match ssh_unit.stop() {
                Ok(_) => (),
                Err(_) => halt("Error while controlling ssh"),
            };

            // Delete SSH keys
            if let Err(err) = Command::new("rm")
                .arg("-f")
                .arg("/etc/ssh/ssh_host_*")
                .status()
            {
                halt(&format!("Failed to delete SSH keys: {}", err));
            }

            // start the sshd service
            match ssh_unit.start() {
                Ok(_) => (),
                Err(_) => halt("Failed to restart the sshd service"),
            };

            // Creating a new manifest
            let ais_result: UnifiedErrorResult<AisInfo> = UnifiedErrorResult::new(AisInfo::new());
            let mut ais_data: AisInfo = ais_result.unwrap();
            ais_data.machine_id = Some(
                truncate(
                    &create_hash(format!(
                        "{}{}",
                        &ais_data
                            .clone()
                            .machine_ip
                            .unwrap_or(String::from("10.1.0.255")),
                        &ais_data
                            .clone()
                            .machine_id
                            .unwrap_or(String::from("00:00:00:00:00"))
                    )),
                    16,
                )
                .to_owned(),
            );

            let _ = ais_data.create_manifest();
            //  Generating the new hostname

            #[allow(unused_assignments)]
            let mut new_hostname = String::new();
            new_hostname = format!("ais_{}.local", ais_data.machine_id.expect("0000000000000000"));

            // Attempt to set the new hostname
            match set(new_hostname.clone()) {
                Ok(()) => {
                    // Regester it on the network 
                    let output = Command::new("/sbin/dhclient")
                    .output()
                    .expect("Failed to execute command");
                    match output.status.success() {
                        true => println!("Hostname set successfully to: {}", new_hostname),
                        false => halt("Error setting hostname")
                    }
                }
                Err(err) => halt(&format!("Failed to set hostname: {}", err)),
            }

            // * we have to disable our server ais_firstrun.service

            match make_file(PathType::Content(installed_path)) {
                Ok(d) => match d {
                    true => notice("Initialized"),
                    false => halt("Loop time"),
                },
                Err(e) => halt(&format!("{}", e)),
            };
            
        }
    }
}
