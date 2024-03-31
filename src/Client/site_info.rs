use std::{
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use system::{create_hash, errors::SystemError, path_present, truncate, PathType};
use shared::{errors::{AisError, UnifiedError}, git_data::GitAuth};
use crate::git_actions::GitAction;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Updates {
    UpToDate,
    OutOfDate,
}

#[derive(Clone, Debug)]
pub struct SiteInfo {
    pub application_folder: PathType,
    pub application_status: Updates,
}

impl SiteInfo {
    pub fn new(
        git_cred: Arc<RwLock<GitAuth>>,
    ) -> Result<Self, UnifiedError> {
        let git_creds: RwLockReadGuard<'_, GitAuth> = match git_cred.read() {
            Ok(d) => d,
            Err(_) => {
                return Err(UnifiedError::from_ais_error(AisError::ThreadedDataError(Some(
                    String::from("Git Creds"),
                ))))
            }
        };

        let application_folder = PathType::PathBuf(Self::get_site_folder(&git_creds)?);

        let check_remote_ahead_action = GitAction::CheckRemoteAhead(application_folder.clone());
        let application_status: Updates = match check_remote_ahead_action.execute() {
            Ok(is_ahead) => match is_ahead {
                true => Updates::OutOfDate,
                false => Updates::UpToDate,
            },
            Err(e) => return Err(e),
        };

        Ok(Self {
            application_folder,
            application_status,
        })
    }

    pub fn get_site_folder(
        git_auth: &RwLockReadGuard<'_, GitAuth>,
    ) -> Result<PathBuf, UnifiedError> {
        let site_folder_string: String = format!("{}-{}", git_auth.user, git_auth.repo,);

        let site_folder: String = truncate(&create_hash(site_folder_string), 8).to_owned();

        let site_path: String = format!("/var/www/current/{}", site_folder);
        // sanity check

        match path_present(&PathType::Content(site_path.clone())) {
            Ok(d) => match d {
                true => return Ok(PathBuf::from(site_path.clone())),
                false => {
                    return Err(UnifiedError::from_system_error(SystemError::new_details(
                        system::errors::SystemErrorType::ErrorCreatingDir,
                        &format!("Dir: {} not found", site_path.clone()),
                    )))
                }
            },
            Err(e) => return Err(UnifiedError::from_system_error(e)),
        }
    }
}
