use pretty::{dump, notice};
use shared::{
    errors::{ErrorInfo, UnifiedError}, git_actions, git_data::{GitAuth, GitCredentials}, site_info::SiteInfo
};
use system::{create_hash, make_dir, truncate, ClonePath, PathType, SystemError};

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
                                let action = git_actions::GitAction::Clone { repo_url: format!("https://github.com/{}/{}.git", git_auth.user, git_auth.repo), destination: ais_progect_path };
                                match action.execute() {
                                    Ok(_) => return Ok(()),
                                    Err(e) => return Err(e),
                                }
                            },
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
        Ok(_) => println!("Directories created successfully"),
        Err(err) => eprintln!("Error creating directories: {:?}", err),
    }
}
