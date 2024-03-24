use std::{
    os::unix::process::ExitStatusExt, path::PathBuf, process::{Command, ExitStatus}
};
use system::path_present;
use shared::errors::{AisError, GitError, GitWarning, UnifiedError, UnifiedWarning};

/// Function to check if Git is installed
fn check_git_installed() -> Result<(), UnifiedError> {
    let output: std::process::Output = match Command::new("git").arg("--version").output() {
        Ok(output) => output,
        Err(io_err) => return Err(UnifiedError::AisError(AisError::new(&io_err.to_string()))),
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(UnifiedError::GitError(GitError::GitNotInstalled))
    }
}

/// Enum representing Git actions
#[derive(Debug)]
pub enum GitAction {
    Clone {
        repo_url: String,
        destination: PathBuf,
    },
    Pull(PathBuf),
    Push {
        directory: PathBuf,
    },
    Stage {
        directory: PathBuf,
        files: Vec<String>,
    },
    Commit {
        directory: PathBuf,
        message: String,
    },
    CheckRemoteAhead(PathBuf),
}

impl GitAction {
    pub fn execute(&self) -> Result<(bool, Vec<UnifiedWarning>), UnifiedError> {
        check_git_installed()?;
        match self {
            GitAction::Clone {
                repo_url,
                destination,
            } => {
                path_present(destination.to_path_buf())?;
                execute_git_command(&["clone", repo_url, destination.to_str().unwrap()])
            }
            GitAction::Pull(directory) => {
                path_present(directory.to_path_buf())?;
                execute_git_command_with_warnings(&["-C", directory.to_str().unwrap(), "pull"])
            }
            GitAction::Push { directory } => {
                path_present(directory.to_path_buf())?;
                execute_git_command_with_warnings(&["-C", directory.to_str().unwrap(), "push"])
            }
            GitAction::Stage { directory, files } => {
                path_present(directory.to_path_buf())?;
                let mut args = vec!["-C", directory.to_str().unwrap(), "add"];
                args.extend(files.iter().map(|s| s.as_str()));
                execute_git_command_with_warnings(&args)
            }
            GitAction::Commit { directory, message } => {
                path_present(directory.to_path_buf())?;
                execute_git_command_with_warnings(&[
                    "-C",
                    directory.to_str().unwrap(),
                    "commit",
                    "-m",
                    message,
                ])
            }
            GitAction::CheckRemoteAhead(directory) => {
                path_present(directory.to_path_buf())?;
                check_remote_ahead(directory)
            }
        }
    }
}

// Function to execute a Git command and capture warnings
fn execute_git_command_with_warnings(
    args: &[&str],
) -> Result<(bool, Vec<UnifiedWarning>), UnifiedError> {
    let output: std::process::Output = match Command::new("git").args(args).output() {
        Ok(output) => output,
        Err(io_err) => return Err(UnifiedError::AisError(AisError::new(&io_err.to_string()))),
    };

    let success: bool = output.status.success();
    let warnings: Vec<UnifiedWarning> = extract_warnings(&output.stderr);

    if success {
        Ok((true, warnings))
    } else {
        Err(UnifiedError::GitError(GitError::CommandFailed(
            output.status,
        )))
    }
}

// Function to extract warnings from Git output
fn extract_warnings(stderr: &[u8]) -> Vec<UnifiedWarning> {
    String::from_utf8_lossy(stderr)
        .lines()
        .filter_map(|line| match line {
            line if line.contains("LF will be replaced by CRLF") => {
                Some(UnifiedWarning::GitWarning(GitWarning::LineEndingConversion))
            }
            line if line.contains("warning: unable to access") => {
                Some(UnifiedWarning::GitWarning(GitWarning::PermissionDenied))
            }
            line if line.contains("warning: file size exceeds") => Some(
                UnifiedWarning::GitWarning(GitWarning::FileSizeExceedsThreshold),
            ),
            // Add more warning checks here as needed
            _ => None,
        })
        .collect()
}

// Function to execute a Git command
fn execute_git_command(args: &[&str]) -> Result<(bool, Vec<UnifiedWarning>), UnifiedError> {
    let output: std::process::Output = match Command::new("git").args(args).output() {
        Ok(output) => output,
        Err(io_err) => return Err(UnifiedError::AisError(AisError::new(&io_err.to_string()))),
    };

    if output.status.success() {
        let warnings: Vec<UnifiedWarning> = extract_warnings(&output.stderr);
        Ok((true, warnings))
    } else {
        Err(UnifiedError::GitError(GitError::CommandFailed(
            output.status,
        )))
    }
}

// Function to check if the remote repository is ahead of the local repository
fn check_remote_ahead(directory: &PathBuf) -> Result<(bool, Vec<UnifiedWarning>), UnifiedError> {
    let fetch_output: (bool, Vec<UnifiedWarning>) = execute_git_command(&["-C", directory.to_str().unwrap(), "fetch"])?;

    if !fetch_output.0 {
        return Err(UnifiedError::GitError(GitError::CommandFailed(
            ExitStatus::from_raw(1),
        )));
    }

    let local_hash: String =
        execute_git_hash_command(&["-C", directory.to_str().unwrap(), "rev-parse", "@"])?;
    let remote_hash: String =
        execute_git_hash_command(&["-C", directory.to_str().unwrap(), "rev-parse", "@{u}"])?;

    Ok((remote_hash != local_hash, Vec::new()))
}

// Function to execute a Git hash command
fn execute_git_hash_command(args: &[&str]) -> Result<String, UnifiedError> {
    let output: std::process::Output = match Command::new("git").args(args).output() {
        Ok(output) => output,
        Err(io_err) => return Err(UnifiedError::AisError(AisError::new(&io_err.to_string()))),
    };

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(UnifiedError::GitError(GitError::CommandFailed(
            output.status,
        )))
    }
}
