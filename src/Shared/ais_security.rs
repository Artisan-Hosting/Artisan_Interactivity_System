use std::thread;

use crate::errors::{AisError, Caller, ErrorInfo, Severity};
#[allow(unused_imports)]
use crate::{
    ais_data::{AisInfo, AisVersion},
    errors::{UnifiedError, UnifiedErrorResult},
    git_data::GitCredentials,
};
use pretty::notice;
use system::SystemError;
use systemstat::Duration;

pub fn check_cf() -> Result<bool, UnifiedError> {
    // * Put the appilcation IN a hold state if no credential file is found
    match GitCredentials::new() {
        Ok(_) => return Ok(true), // true means We ok
        Err(e) => match e {
            // ? We look for a system error saying we could not find the artiisan.cf file.
            // ? This means that the system has been initialized but no clients have been
            // ? Registered. Theres is not point in running loops or monitoring when the
            // ? Server is not in a usable state
            UnifiedError::SystemError(k, d) => match d.kind {
                system::errors::SystemErrorType::ErrorOpeningFile => {
                    notice("Awating registration");
                    thread::sleep(Duration::from_secs_f32(30.0));
                    return Ok(false); // false means that we should exit because the file was not found
                }
                _ => return Err(UnifiedError::SystemError(k, SystemError::new(d.kind))),
            },
            e => return Err(e),
        },
    };
}

pub fn check_manifest(ais: AisInfo) -> Result<(), UnifiedError> {
    let manifest_version: AisVersion = ais.system_version;
    let system_version: AisVersion = AisInfo::current_version();

    match manifest_version == system_version {
        true => Ok(()),
        false => Err(UnifiedError::AisError(
            ErrorInfo::with_severity(
                Caller::Function(true, Some("Check Manifest".to_owned())),
                Severity::Warning,
            ),
            AisError::InvalidManifest(Some("Manifest Version".to_owned())),
        )),
    }
}

#[test]
fn test_cf() {
    // Just ensure it returns something
    assert!(check_cf().is_ok() || check_cf().is_err())
}

#[test]
fn test_version_match() {
    // ? This ensures that the version we are expecting is the same one we'll create
    let ais: AisInfo = UnifiedErrorResult::new(AisInfo::new()).unwrap();
    assert_eq!(ais.system_version, AisInfo::current_version())
}
