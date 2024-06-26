use lsb_release::LsbRelease;
use pretty::output;
use shared::ais_data;
use systemstat::{Platform, System};

fn main() {
    let sys: System = System::new();
    let ais_info: ais_data::AisInfo = ais_data::AisInfo::new().unwrap();

    let system_mem: String = match sys.memory() {
        Ok(mem) => {
            let used_memory: u64 = mem.total.as_u64() - mem.free.as_u64();
            let percentage_used: f64 = (used_memory as f64 / mem.total.as_u64() as f64) * 100.0;
            format!("{}", percentage_used)
        }
        Err(x) => format!("\nMemory: error: {}", x),
    };

    let lsb_failsafe: LsbRelease = LsbRelease {
        id: String::from("failsafe"),
        desc: String::from("System in a damanged state"),
        version: String::from("4.20"),
        code_name: String::from("Wacky Whitfield"),
    };

    let ais_version = ais_info.system_version;
    let ais_identyfi: String = ais_info
        .machine_id
        .unwrap_or(String::from("error parsing manifest"));
    let system_version = lsb_release::info().unwrap_or(lsb_failsafe);
    let system_hostname = gethostname::gethostname();
    let (system_load_1, system_load_5, system_load_15) = match sys.load_average() {
        Ok(l) => (l.one, l.five, l.fifteen),

        Err(_) => {
            let val: f32 = 0.0;
            (val, val, val)
        }
    };

    let welcome_text = format!(
        r#"
                  _    _                         _    _                   _
     /\          | |  (_)                       | |  | |                 (_) 
    /  \    _ __ | |_  _  ___   __ _  _ __      | |__| |  ___   ___ | |_     _ __    __ _
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

This server is hosted by Artisan Hosting. If you're reading this now would probably be a goodtime 
to contact me at dwhitfield@artisanhosting.net or shoot me a text at 414-578-0988. Thank you for
supporting me and Artisan Hosting.

"#,
        format!("{} - {}", system_version.version, system_version.code_name),
        format!("{}_{}", ais_version.version_number.to_string(), ais_version.version_code),
        ais_identyfi.trim_end(),
        system_hostname,
        system_load_1,
        system_load_5,
        system_load_15,
        system_mem
    );

    output("BLUE", &format!("{}", welcome_text));
}
