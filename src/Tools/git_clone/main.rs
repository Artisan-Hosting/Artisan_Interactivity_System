use pretty::{dump, notice};
use shared::{
    errors::{Caller, ErrorInfo, UnifiedError},
    git_actions,
    git_data::{GitAuth, GitCredentials},
    site_info::SiteInfo,
};
use system::{chown_recursive, create_hash, make_dir, truncate, ClonePath, PathType, SystemError};

// Structs representing GitCredentials and GitAuth omitted for brevity

fn create_directories_for_git_auth(git_auth: &GitAuth) -> Result<(), UnifiedError> {
    let site_folder_string: String = format!("{}-{}", git_auth.user, git_auth.repo,);
    let site_folder: String = truncate(&create_hash(site_folder_string), 8).to_owned();
    let ais_progect_path: PathType = PathType::Content(format!("/var/www/current/{}", site_folder));

    match SiteInfo::new(&git_auth) {
        Ok(_) => (),
        Err(e) => match e {
            UnifiedError::SystemError(_, data) => match data.kind {
                system::errors::SystemErrorType::ErrorCreatingDir => {
                    // Create directories recursively if they don't exist
                    match make_dir(ais_progect_path.clone_path()) {
                        Ok(b) => match b {
                            true => {
                                // Once the directory is created we clone the data into it
                                let action = git_actions::GitAction::Clone {
                                    repo_url: format!(
                                        "git@github.com:{}/{}.git",
                                        git_auth.user, git_auth.repo
                                    ),
                                    destination: ais_progect_path.clone_path(),
                                };
                                match action.execute() {
                                    Ok(_) => {
                                        git_actions::GitAction::SetSafe(ais_progect_path.clone_path()).execute()?;
                                        chown_recursive(ais_progect_path.clone(), Some(33), Some(33))?
                                    },
                                    Err(e) => { 
                                        // Repacking error
                                        let err: UnifiedError = match e {
                                            UnifiedError::LoggerError(_, e) => UnifiedError::LoggerError(ErrorInfo::new(Caller::Function(true, Some("Logger Error".to_string()))), e),
                                            UnifiedError::SystemError(_, e) => UnifiedError::SystemError(ErrorInfo::new(Caller::Function(true, Some("System Error".to_string()))), e),
                                            UnifiedError::RecsError(_, e) => UnifiedError::RecsError(ErrorInfo::new(Caller::Function(true, Some("Recs Error".to_string()))), e),
                                            UnifiedError::GitError(_, e) => UnifiedError::GitError(ErrorInfo::new(Caller::Function(true, Some("Git action execute".to_string()))), e),
                                            UnifiedError::AisError(_, e) => UnifiedError::AisError(ErrorInfo::new(Caller::Function(true, Some("AIS error".to_string()))), e),
                                        };
                                        return Err(err)
                                    },
                                }
                            }
                            false => {
                                dump("error while making dirs");
                                panic!()
                            }
                        },
                        Err(e) => return Err(UnifiedError::from_system_error(e)),
                    }
                }
                e => {
                    return Err(UnifiedError::SystemError(
                        ErrorInfo::new(shared::errors::Caller::Function(false, None)),
                        SystemError::new(e),
                    ))
                }
            },
            e => return Err(e),
        },
    };

    notice(&ais_progect_path.to_string());
    // let git_progect_path: PathType = site_data.application_folder;

    // Create directories recursively if they don't exist
    match make_dir(ais_progect_path) {
        Ok(b) => match b {
            true => return Ok(()),
            false => {
                dump("error while making dirs");
                panic!()
            }
        },
        Err(e) => return Err(UnifiedError::from_system_error(e)),
    }
}

fn create_directories_for_git_credentials(
    credentials: &GitCredentials,
) -> Result<(), UnifiedError> {
    for auth in &credentials.auths {
        create_directories_for_git_auth(auth)?;
    }
    Ok(())
}

fn main() {
    // Load GitCredentials from file
    let credentials = match GitCredentials::new() {
        Ok(creds) => creds,
        Err(err) => {
            eprintln!("Error loading GitCredentials: {:?}", err);
            return;
        }
    };

    // Create directories for each GitAuth entry
    match create_directories_for_git_credentials(&credentials) {
        Ok(_) => notice("Directories created successfully"),
        Err(err) => dump(&format!("Error creating directories: {:?}", err)),
    }
}
