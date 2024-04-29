use rustpython_vm::pymodule;

/// HELLO, Much like **RustPython** itself this features is *HIGHLY DEVELOPMENTAL* The goal of this section
/// Is to access some complex data and functions from the Shared Lib in a quick and accessible language (Python)
/// The intention is to allow for quicker P.O.C to be built by using python intead of rust. Second some tooling
/// like packaging progects into apacs can be done with easier in python than rust. Also system tools like
/// Potentially viewing and modifying `Artisan Manifests` or `Artisan Credentials` could be accomplished
/// in a simplme language while leveraging the heavy lifting of the encryption functions and the socket communication
/// already done in rust. When fully mature we could allow this to be the entry point for client ssh connections with the
/// system, Allowing for a intermediate not directly offering Virtual Machines but allowing clients to access a machine and
/// make changes to services that they run while leaving services for any other clients untouched. But this is just a small
/// Proof of concept that could be a dumb idea.
fn main() {
    rustpython::run(|vm| {
        vm.add_native_module("ais".to_owned(), Box::new(artisan::make_module));
    });
}

#[pymodule]
mod artisan {
    use pretty::{notice, output};
    use rustpython_vm::builtins::PyStrRef;
    use shared::{
        ais_data::AisInfo,
        emails::{Email, EmailSecure},
        encrypt::Commands,
        errors::UnifiedErrorResult,
    };

    fn get_ais_info() -> AisInfo {
        let d = AisInfo::new().unwrap();
        return d;
    }

    #[pyfunction]
    fn get_hostname() -> String {
        let ais_data: AisInfo = get_ais_info();
        let machine_id = ais_data.machine_id.unwrap_or("0000000".to_owned());
        let hostname: String = format!("ais_{}.local", machine_id);
        return hostname;
    }

    #[pyfunction]
    fn version() -> String {
        let ais_data: AisInfo = get_ais_info();
        let version_struct = ais_data.system_version;
        let codename: &str = match version_struct.version_code {
            shared::ais_data::AisCode::Production => "Prod",
            shared::ais_data::AisCode::ProductionCandidate => "RC",
            shared::ais_data::AisCode::Beta => "Beta",
            shared::ais_data::AisCode::Alpha => "Alpha",
        };

        return format!(
            "Artisan Interactivity System: {}_{}",
            version_struct.version_number, codename
        );
    }

    #[pyfunction]
    fn send_email(subject: PyStrRef, body: PyStrRef) -> bool {
        let message: Email = Email {
            subject: subject.to_string(),
            body: body.to_string(),
        };

        let message_secure: EmailSecure =
            UnifiedErrorResult::new(EmailSecure::new(message)).unwrap();

        match message_secure.send() {
            Ok(_) => return true,
            Err(e) => {
                output("RED", &format!("Unified error: {}", e));
                return false;
            }
        }
    }

    #[pyfunction]
    fn encrypt_text(data: PyStrRef) -> Option<String> {
        let command = Commands::EncryptText(data.to_string());
        match command.execute() {
            Ok(d) => return d,
            Err(err) => {
                output("RED", &format!("Unified error: {}", err));
                return None;
            }
        }
    }

    #[pyfunction]
    fn decrypt_text(data: PyStrRef) -> Option<String> {
        let command = Commands::DecryptText(data.to_string());
        match command.execute() {
            Ok(d) => return d,
            Err(err) => {
                output("RED", &format!("Unified error: {}", err));
                return None;
            }
        }
    }

    // #[pyfunction]
    // fn initialize_dusa() -> bool {
    //     let dusa_initializing: Dusa = Dusa::initialize();
    // }

    // #[pyfunction]
    // fn encrypt_text(data: PyStrRef) -> String {

    // }

    #[pyfunction]
    fn error_test() {
        panic!()
    }

    #[pyfunction]
    fn pretty_test() {
        notice("hello from python !");
    }

    #[pyfunction]
    fn debug_print() {
        let ais_data: AisInfo = get_ais_info();
        ais_data.print_all();
    }
}

#[pymodule]
mod system {
    
}