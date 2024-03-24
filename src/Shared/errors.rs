use std::{fmt, io, process::ExitStatus};
use logging::errors::LoggerError;
use recs::errors::{RecsError, RecsWarning};
use system::errors::SystemError;

/// Enum representing unified warnings that can occur in the system.
#[derive(Debug)]
pub enum UnifiedWarning {
    /// Recs warnings.
    RecsWarning(RecsWarning),
    /// Ais warnings.
    AisWarning(AisWarning),
    /// Git warnings.
    GitWarning(GitWarning),
}

/// Enum representing Git warnings.
#[derive(Debug)]
pub enum GitWarning {
    /// Line ending conversion warning.
    LineEndingConversion,
    /// Permission denied warning.
    PermissionDenied,
    /// File size exceeds threshold warning.
    FileSizeExceedsThreshold,
    // Add more warning variants as needed
}

/// Enum representing Ais warnings.
#[derive(Debug)]
pub enum AisWarning {
    /// Generic Ais warning.
    Generic,
}

/// Enum representing unified errors that can occur in the system.
#[derive(Debug)]
pub enum UnifiedError {
    /// Logger errors.
    LoggerError(LoggerError),
    /// System errors.
    SystemError(SystemError),
    /// RECS errors.
    RecsError(RecsError),
    /// Git errors.
    GitError(GitError),
    /// Ais errors.
    AisError(AisError),
}

impl fmt::Display for UnifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnifiedError::LoggerError(e) => write!(f, "Logger error: {}", e),
            UnifiedError::SystemError(e) => write!(f, "System error: {}", e),
            UnifiedError::RecsError(e) => write!(f, "RECS error: {}", e),
            UnifiedError::GitError(e) => write!(f, "Git error: {}", &e.description()),
            UnifiedError::AisError(e) => write!(f, "AIS error: {}", &e.description()),
        }
    }
}

// Implement conversions from specific error types to UnifiedError
impl From<LoggerError> for UnifiedError {
    fn from(error: LoggerError) -> UnifiedError {
        UnifiedError::LoggerError(error)
    }
}

impl From<SystemError> for UnifiedError {
    fn from(error: SystemError) -> UnifiedError {
        UnifiedError::SystemError(error)
    }
}

impl From<RecsError> for UnifiedError {
    fn from(error: RecsError) -> UnifiedError {
        UnifiedError::RecsError(error)
    }
}

impl From<GitError> for UnifiedError {
    fn from(error: GitError) -> UnifiedError {
        UnifiedError::GitError(error)
    }
}

impl From<AisError> for UnifiedError {
    fn from(error: AisError) -> UnifiedError {
        UnifiedError::AisError(error)
    }
}

/// Helper method to get the description of a custom error.
impl UnifiedError {
    pub fn description(&self) -> String {
        match self {
            UnifiedError::LoggerError(e) => format!("{:?}", e.details),
            UnifiedError::SystemError(e) => format!("{:?}", e.details),
            UnifiedError::RecsError(e) => format!("{:?}", e.details),
            UnifiedError::GitError(e) => e.description().to_owned(),
            UnifiedError::AisError(e) => e.description().to_owned(),
        }
    }
}

/// Implementation for Ais errors.
#[derive(Debug)]
pub enum AisError {
    /// SSH flagged user error.
    SshFlaggedUser(Option<String>),
    /// SSH unknown user error.
    SshUnknownUser(Option<String>),
    /// SSH unflagged user error.
    SshUnflaggedUser(Option<String>),
    /// Threaded data access error.
    ThreadedDataError(Option<String>),
    /// Threaded data not populated error.
    ThreadedDataNotPopulated(Option<String>),
    /// Site information invalid error.
    SiteInfoInvalid(Option<String>),
    /// Site initialization failed error.
    SiteInitializationFailed(Option<String>),
    /// Site setup failed error.
    SiteFailed(Option<String>),
    /// Git commend didn't run sucessfully,
    GitCommandFailed(Option<String>),
    /// Git credentials invalid error.
    GitCredentialsInvalid(Option<String>),
    /// Git credentials unknown error.
    GitCredentialsUnknown(Option<String>),
    /// Git invalid release error.
    GitInvalidRelease(Option<String>),
    /// Git invalid commit error.
    GitInvalidCommit(Option<String>),
    /// Git network error.
    GitNetworkError(Option<String>),
    /// Cryptography failed error.
    CryptFailed(Option<String>),
    /// Update error.
    UpdateError(Option<String>),
    /// Up to date error.
    UpToDate(Option<String>),
    /// Generic system error.
    SystemError(Option<String>),
    /// Encryption not ready error.
    EncryptionNotReady(Option<String>),
}

impl AisError {
    /// Creates a new AisError with the specified description.
    pub fn new(description: &str) -> AisError {
        AisError::SystemError(Some(description.to_string()))
    }

    /// Returns the description of the AisError.
    pub fn description(&self) -> &str {
        match self {
            AisError::SshFlaggedUser(desc)
            | AisError::SshUnknownUser(desc)
            | AisError::SshUnflaggedUser(desc)
            | AisError::ThreadedDataError(desc)
            | AisError::ThreadedDataNotPopulated(desc)
            | AisError::SiteInfoInvalid(desc)
            | AisError::SiteInitializationFailed(desc)
            | AisError::SiteFailed(desc)
            | AisError::GitCommandFailed(desc)
            | AisError::GitCredentialsInvalid(desc)
            | AisError::GitCredentialsUnknown(desc)
            | AisError::GitInvalidRelease(desc)
            | AisError::GitInvalidCommit(desc)
            | AisError::GitNetworkError(desc)
            | AisError::CryptFailed(desc)
            | AisError::UpdateError(desc)
            | AisError::UpToDate(desc)
            | AisError::SystemError(desc)
            | AisError::EncryptionNotReady(desc) => desc.as_deref().unwrap_or("An unspecified error occurred"),
        }
    }
}

/// Enum representing Git errors.
#[derive(Debug)]
pub enum GitError {
    /// Git command failed error.
    CommandFailed(ExitStatus),
    /// IO error.
    IoError(io::Error),
    /// UTF-8 error.
    Utf8Error(std::str::Utf8Error),
    /// Git warning.
    Warning(GitWarning),
    /// Git not installed error.
    GitNotInstalled,
}

impl GitError {
    /// Returns the description of the GitError.
    pub fn description(&self) -> &str {
        match self {
            GitError::CommandFailed(_) => "Git command failed",
            GitError::IoError(_) => "IO error",
            GitError::Utf8Error(_) => "UTF-8 error",
            GitError::Warning(_) => "Git warning",
            GitError::GitNotInstalled => "Git is not installed",
        }
    }
}