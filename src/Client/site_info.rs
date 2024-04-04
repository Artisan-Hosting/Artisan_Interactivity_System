//! # Site Information Module
//!
//! This module defines structures and functions related to site information.

use std::{
    path::PathBuf,
    sync::{Arc, RwLock, RwLockReadGuard},
};

use crate::git_actions::GitAction;
use shared::{
    errors::{Caller, UnifiedError},
    git_data::{GitAuth, GitCredentials},
};
use system::{create_hash, errors::SystemError, path_present, truncate, PathType};

/// Enum representing the update status of a site.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Updates {
    /// The site is up to date.
    UpToDate,
    /// The site is out of date and needs updates.
    OutOfDate,
}

/// Struct holding information about a site.
#[derive(Clone, Debug)]
pub struct SiteInfo {
    /// The folder where the site's application resides.
    pub application_folder: PathType,
    /// The status of the site's application.
    pub application_status: Updates,
}

impl SiteInfo {
    /// Creates a new SiteInfo instance.
    ///
    /// # Arguments
    ///
    /// * `git_cred` - A reference-counted lock containing Git credentials.
    ///
    /// # Returns
    ///
    /// A Result containing the new SiteInfo instance if successful, or an error.
    pub fn new(git_creds: &GitAuth) -> Result<Self, UnifiedError> {
        let mut results: Vec<Self> = Vec::new();

        let application_folder = PathType::PathBuf(Self::get_site_folder(&git_creds)?);

        let check_remote_ahead_action = GitAction::CheckRemoteAhead(application_folder.clone());
        let application_status: Updates = match check_remote_ahead_action.execute() {
            Ok(is_ahead) => match is_ahead {
                true => Updates::OutOfDate,
                false => Updates::UpToDate,
            },
            Err(e) => return Err(e),
        };

        let git_cred_data = Self {
            application_folder,
            application_status,
        };

        return Ok(git_cred_data);
    }

    /// Retrieves the path to the site folder.
    ///
    /// # Arguments
    ///
    /// * `git_auth` - A read guard containing Git authentication information.
    ///
    /// # Returns
    ///
    /// A Result containing the path to the site folder if successful, or an error.
    pub fn get_site_folder(git_auth: &GitAuth) -> Result<PathBuf, UnifiedError> {
        let site_folder_string: String = format!("{}-{}", git_auth.user, git_auth.repo,);

        let site_folder: String = truncate(&create_hash(site_folder_string), 8).to_owned();

        let site_path: String = format!("/var/www/current/{}", site_folder);

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

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::{Arc, RwLock};

//     #[test]
//     fn test_site_info_creation() {
//         // Mocking GitAuth data
//         let git_auth = Arc::new(RwLock::new(GitAuth::new_mock("user", "repo")));

//         // Creating a new SiteInfo instance
//         let site_info_result = SiteInfo::new(git_auth.clone());

//         // Asserting that the SiteInfo instance was created Incorrectly so we can only work in the assigned dir
//         assert!(site_info_result.is_err());
//     }

// }
