use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, ExitStatus},
};

use crate::errors::{AisError, GitError, UnifiedError};
use system::{path_present, PathType};

/// Function to check if Git is installed.
fn check_git_installed() -> Result<(), UnifiedError> {
    let output: std::process::Output = match Command::new("git").arg("--version").output() {
        Ok(output) => output,
        Err(io_err) => {
            return Err(UnifiedError::from_ais_error(AisError::new(
                &io_err.to_string(),
            )))
        }
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(UnifiedError::from_git_error(GitError::GitNotInstalled))
    }
}

/// Enum representing Git actions.
#[derive(Debug)]
pub enum GitAction {
    Clone {
        repo_url: String,
        destination: PathType,
    },
    Pull(PathType),
    Push {
        directory: PathType,
    },
    Stage {
        directory: PathType,
        files: Vec<String>,
    },
    Commit {
        directory: PathType,
        message: String,
    },
    CheckRemoteAhead(PathType),
}

impl GitAction {
    /// Execute the Git action.
    pub fn execute(&self) -> Result<bool, UnifiedError> {
        check_git_installed()?;
        match self {
            GitAction::Clone {
                repo_url,
                destination,
            } => {
                path_present(destination)?;
                execute_git_command(&["clone", repo_url, destination.to_str().unwrap()])
            }
            GitAction::Pull(directory) => {
                path_present(directory)?;
                execute_git_command(&["-C", directory.to_str().unwrap(), "pull"])
            }
            GitAction::Push { directory } => {
                path_present(directory)?;
                execute_git_command(&["-C", directory.to_str().unwrap(), "push"])
            }
            GitAction::Stage { directory, files } => {
                path_present(directory)?;
                let mut args = vec!["-C", directory.to_str().unwrap(), "add"];
                args.extend(files.iter().map(|s| s.as_str()));
                execute_git_command(&args)
            }
            GitAction::Commit { directory, message } => {
                path_present(directory)?;
                execute_git_command(&[
                    "-C",
                    directory.to_str().unwrap(),
                    "commit",
                    "-m",
                    message,
                ])
            }
            GitAction::CheckRemoteAhead(directory) => {
                path_present(directory)?;
                check_remote_ahead(directory)
            }
        }
    }
}

/// Execute a Git command.
fn execute_git_command(args: &[&str]) -> Result<bool, UnifiedError> {
    let output: std::process::Output = match Command::new("git").args(args).output() {
        Ok(output) => output,
        Err(io_err) => {
            return Err(UnifiedError::from_ais_error(AisError::new(
                &io_err.to_string(),
            )))
        }
    };

    if output.status.success() {
        Ok(true)
    } else {
        Err(UnifiedError::from_git_error(GitError::CommandFailed(
            output.status,
        )))
    }
}

/// Check if the remote repository is ahead of the local repository.
fn check_remote_ahead(directory: &PathType) -> Result<bool, UnifiedError> {
    let fetch_output: bool = execute_git_command(&["-C", directory.to_str().unwrap(), "fetch"])?;

    if !fetch_output {
        return Err(UnifiedError::from_git_error(GitError::CommandFailed(
            ExitStatus::from_raw(1),
        )));
    }

    let local_hash: String =
        execute_git_hash_command(&["-C", directory.to_str().unwrap(), "rev-parse", "@"])?;
    let remote_hash: String =
        execute_git_hash_command(&["-C", directory.to_str().unwrap(), "rev-parse", "@{u}"])?;

    Ok(remote_hash != local_hash)
}

/// Execute a Git hash command.
fn execute_git_hash_command(args: &[&str]) -> Result<String, UnifiedError> {
    let output: std::process::Output = match Command::new("git").args(args).output() {
        Ok(output) => output,
        Err(io_err) => {
            return Err(UnifiedError::from_ais_error(AisError::new(
                &io_err.to_string(),
            )))
        }
    };

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(UnifiedError::from_git_error(GitError::CommandFailed(
            output.status,
        )))
    }
}

#[cfg(feature = "git")]
#[cfg(test)]
mod tests {
    use system::del_dir;

    use super::*;
    use std::fs;

    const TEST_REPO_URL: &str = "https://github.com/Artisan-Hosting/dummy.git";
    const TEST_DESTINATION: &str = "/tmp/test_repo";

    #[test]
    fn test_check_git_installed() {
        // Assuming Git is installed on the system
        assert!(check_git_installed().is_ok());

        // Assuming Git is not installed on the system
        // Uninstall Git before running this test
        // assert!(check_git_installed().is_err());
    }

    #[test]
    fn test_git_clone() {
        let _ = del_dir(&PathType::Content(TEST_REPO_URL.to_string()));
        let _result = GitAction::Clone {
            repo_url: TEST_REPO_URL.to_string(),
            destination: PathType::Content(TEST_DESTINATION.to_string()),
        }
        .execute();
        // assert!(result.is_ok());
        assert!(fs::metadata(TEST_DESTINATION).is_ok());
    }

    #[test]
    fn test_git_pull() {
        let result = GitAction::Pull(PathType::Content(TEST_DESTINATION.to_string())).execute().unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_check_remote_ahead() {
        // Assuming Git is configured with a remote repository
        let result = GitAction::CheckRemoteAhead(PathType::Content(TEST_DESTINATION.to_string()))
            .execute();
        assert!(result.is_ok());
    }
}
