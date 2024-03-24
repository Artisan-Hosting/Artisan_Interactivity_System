use std::{fs::File, io::Read};

use system::is_path;
use systemstat::{Platform, System};
use shared::ais_data;

fn main() {
    let sys = System::new();
    let ais_info = ais_data::AisInfo::new();

    let system_mem: String = match sys.memory() {
        Ok(mem) => {
            let used_memory: u64 = mem.total.as_u64() - mem.free.as_u64();
            let percentage_used: f64 = (used_memory as f64 / mem.total.as_u64() as f64) * 100.0;
            format!("{}", percentage_used)
        }
        Err(x) => format!("\nMemory: error: {}", x),
    };

    let ais_version = "3.0";
    let ais_identyfi: String = fetch_machine_id();
    let system_info = sys_info::linux_os_release().unwrap();
    let system_version = system_info.pretty_name.unwrap_or_default();
    let system_hostname = hostname::get().unwrap_or_default();
    let (system_load_1, system_load_5, system_load_15) = match sys.load_average() {
        Ok(l) => {
            (l.one, l.five, l.fifteen)
        },

        Err(_) => { 
            let val: f32 = 0.0;
            (val, val, val)
        },
    };


let welcome_text = format!(
        r#"
                  _    _                         _    _
     /\          | |  (_)                       | |  (_)
    /  \    _ __ | |_  _  ___   __ _  _ __      | |__| |  ___   ___ | |_  _  _ __    __ _
   / /\ \  | '__|| __|| |/ __| / _` || '_ \     | '__' | / _ \ /`__|| __|| || '_ \  / _` |
  / ____ \ | |   | |_ | |\__ \| (_| || | | |    | |  | || (_) |\__ \| |_ | || | | || (_| |
 /_/    \_\|_|    \__||_||___/ \__,_||_| |_|    |_|  |_| \___/ |___/ \__||_||_| |_| \__, |
                                                                                     __/ |
                                                                                    |___/   


Your machine at a glance:

Os Version   : {}
AIS Version  : {}
AIS id       : {}
Hostname     : {:?}
System Load  : {:.2}, {:.2}, {:.2}
Mem Usage    : {:.4}%

Welcome!

This server is hosted by Artisan Hosting. If you have any questions or need help,
please don't hesitate to contact me at dwhitfield@artisanhosting.net or shoot me a text at 414-578-0988.
"#,
        system_version, ais_version, ais_identyfi.trim_end(), system_hostname, system_load_1, system_load_5, system_load_15, system_mem
    );

    println!("{}", welcome_text);
}