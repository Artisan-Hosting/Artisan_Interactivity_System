use pretty::dump;
use shared::{errors::UnifiedError, git_data::{GitAuth, GitCredentials}, site_info::SiteInfo};
use system::{make_dir, PathType};

// Structs representing GitCredentials and GitAuth omitted for brevity

fn create_directories_for_git_auth(git_auth: &GitAuth) -> Result<(), UnifiedError> {

    let site_data: SiteInfo = SiteInfo::new(git_auth).unwrap();
    let path: PathType = PathType::Content(format!("{}", site_data.application_folder));

    // Create directories recursively if they don't exist
    match make_dir(path) {
        Ok(b) => match b {
            true => return Ok(()),
            false => {
                dump("error while making dirs");
                panic!()
            },
        },
        Err(e) => return Err(UnifiedError::from_system_error(e)),
    }
}

fn create_directories_for_git_credentials(credentials: &GitCredentials) -> Result<(), UnifiedError> {
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
