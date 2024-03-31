use chrono::{DateTime, Utc};
use logging::errors::LoggerError;
use pretty::output;
use recs::errors::RecsError;
use std::{fmt, io, process::ExitStatus, str::Utf8Error};
use system::errors::SystemError;

/// Enum representing the severity level of an error.
#[derive(Debug, Clone)]
pub enum Severity {
    /// Indicates a fatal error, causing the program to terminate.
    Fatal,
    /// Indicates a non-fatal error, which might be recoverable, but exiting the program might be appropriate.
    NotFatal,
    /// Indicates a warning that should be addressed but can be handled without exiting the program.
    Warning,
}

/// Enum representing different types of timestamps.
///
/// This enum defines two variants:
/// - `CurrentTime`: Represents the current system time.
/// - `_Custom`: Represents a custom timestamp specified by `DateTime<Utc>`.
#[derive(Debug, Clone)]
pub enum TimestampType {
    /// Represents the current system time.
    CurrentTime(DateTime<Utc>),
    /// Represents a custom timestamp.
    _Custom(DateTime<Utc>),
}

/// Enum representing different callers that generate errors.
///
/// This enum categorizes callers into three types:
/// - `Impl`: Represents errors originating from an implementation.
/// - `Function`: Represents errors originating from a function.
/// - `Library`: Represents errors originating from a library.
#[derive(Debug, Clone)]
pub enum Caller {
    /// Represents errors originating from an implementation.
    Impl(bool, Option<String>),
    /// Represents errors originating from a function.
    Function(bool, Option<String>),
    /// Represents errors originating from a library.
    Library(bool, Option<String>),
}

/// Struct containing information about an error.
///
/// This struct encapsulates essential information about an error, including the timestamp of occurrence,
/// the caller that generated the error, and its severity level.
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    /// The timestamp when the error occurred.
    ///
    /// It can be either the current system time or a custom timestamp.
    pub timestamp: TimestampType,
    /// The caller that generated the error.
    ///
    /// Callers are categorized into different types, such as `Impl`, `Function`, or `Library`.
    pub caller: Caller,
    /// The severity level of the error.
    ///
    /// Severity indicates the impact or seriousness of the error, ranging from fatal to warning.
    pub severity: Severity,
}

// Implementation for creating a new timestamp based on the selected type
impl TimestampType {
    /// Creates a new timestamp based on the selected type.
    ///
    /// Returns:
    /// - `DateTime<Utc>`: The generated timestamp.
    pub fn create_timestamp(&self) -> DateTime<Utc> {
        match self {
            TimestampType::CurrentTime(dt) => *dt,
            TimestampType::_Custom(dt) => *dt,
        }
    }
}

/// Enum representing unified errors that can occur in the system.
///
/// This enum encompasses various types of errors that may occur within the system. Each variant contains
/// a specific error type along with its severity level.
#[derive(Debug)]
pub enum UnifiedError {
    /// Errors related to logging operations.
    LoggerError(ErrorInfo, LoggerError),
    /// Errors related to system operations.
    SystemError(ErrorInfo, SystemError),
    /// Errors related to RECS (Resource and Energy Constrained Systems).
    RecsError(ErrorInfo, RecsError),
    /// Errors related to Git operations.
    GitError(ErrorInfo, GitError),
    /// Errors related to AIS (Automatic Identification System) operations.
    AisError(ErrorInfo, AisError),
}

/// A wrapper around `Result` where the error type is `UnifiedError`.
pub struct UnifiedErrorResult<T>(Result<T, UnifiedError>);

impl<T> UnifiedErrorResult<T> {
    /// Constructs a new `UnifiedErrorResult` from a `Result`.
    pub fn new(result: Result<T, UnifiedError>) -> Self {
        UnifiedErrorResult(result)
    }

    /// Unwraps the result, panicking if it contains an error.
    ///
    /// # Panics
    ///
    /// Panics with a message containing information about the error if it is `Err`.
    pub fn unwrap(self) -> T {
        // self.0.unwrap()
        match self.0 {
            Ok(d) => d,
            Err(err) => {
                output("RED", &format!("UnifiedError: {}", err.to_string()));
                std::process::exit(700);
            },
        }
    }
}

impl<T, E> From<Result<T, E>> for UnifiedErrorResult<T>
where 
    E: Into<UnifiedError>,
{
    /// Converts a `Result<T, E>` into a `UnifiedErrorResult<T>`.
    fn from(result: Result<T, E>) -> Self {
        UnifiedErrorResult(result.map_err(Into::into))
    }
}

impl<T> std::ops::Deref for UnifiedErrorResult<T> {
    type Target = Result<T, UnifiedError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl ErrorInfo {
    /// Constructs a new `ErrorInfo` instance with the current timestamp and default severity set to `Fatal`.
    ///
    /// Parameters:
    /// - `caller`: The caller that generated the error.
    ///
    /// Returns:
    /// - `ErrorInfo`: A new instance of `ErrorInfo` with the current timestamp and default severity set to `Fatal`.
    pub fn new(caller: Caller) -> Self {
        let timestamp = TimestampType::CurrentTime(Utc::now());
        ErrorInfo {
            timestamp,
            caller,
            severity: Severity::Fatal,
        }
    }

    /// Constructs a new `ErrorInfo` instance with a custom severity level and the current timestamp.
    ///
    /// Parameters:
    /// - `caller`: The caller that generated the error.
    /// - `severity`: The severity level of the error.
    ///
    /// Returns:
    /// - `ErrorInfo`: A new instance of `ErrorInfo` with the current timestamp and custom severity level.
    pub fn with_severity(caller: Caller, severity: Severity) -> Self {
        let timestamp = TimestampType::CurrentTime(Utc::now());
        ErrorInfo {
            timestamp,
            caller,
            severity,
        }
    }
}

/// Implementation of the conversion trait to convert a `LoggerError` into a `UnifiedError`.
///
/// This conversion automatically creates an `ErrorInfo` instance with detailed information about the error,
/// including the current timestamp, default severity set to `Fatal`, and the caller identified as the Logger library.
impl From<LoggerError> for UnifiedError {
    fn from(error: LoggerError) -> UnifiedError {
        let error_info = ErrorInfo::new(Caller::Library(true, Some(String::from("Logger Lib"))));
        UnifiedError::LoggerError(error_info, error)
    }
}

/// Implementation of the conversion trait to convert a `SystemError` into a `UnifiedError`.
///
/// This conversion automatically creates an `ErrorInfo` instance with detailed information about the error,
/// including the current timestamp, default severity set to `Fatal`, and the caller identified as the System library.
impl From<SystemError> for UnifiedError {
    fn from(error: SystemError) -> UnifiedError {
        let error_info = ErrorInfo::new(Caller::Library(true, Some(String::from("System Lib"))));
        UnifiedError::SystemError(error_info, error)
    }
}

/// Implementation of the conversion trait to convert a `RecsError` into a `UnifiedError`.
///
/// This conversion automatically creates an `ErrorInfo` instance with detailed information about the error,
/// including the current timestamp, default severity set to `Fatal`, and the caller identified as the Rust Encryption Code System library.
impl From<RecsError> for UnifiedError {
    fn from(error: RecsError) -> UnifiedError {
        let error_info = ErrorInfo::new(Caller::Library(
            true,
            Some(String::from("Rust Encryption Code System Lib")),
        ));
        UnifiedError::RecsError(error_info, error)
    }
}

/// Implementation of the conversion trait to convert a `GitError` into a `UnifiedError`.
///
/// This conversion automatically creates an `ErrorInfo` instance with detailed information about the error,
/// including the current timestamp and default severity set to `Fatal`.
impl From<GitError> for UnifiedError {
    fn from(error: GitError) -> UnifiedError {
        let error_info = ErrorInfo::new(Caller::Library(false, None));
        UnifiedError::GitError(error_info, error)
    }
}

/// Implementation of the conversion trait to convert an `AisError` into a `UnifiedError`.
///
/// This conversion automatically creates an `ErrorInfo` instance with detailed information about the error,
/// including the current timestamp and default severity set to `Fatal`.
impl From<AisError> for UnifiedError {
    fn from(error: AisError) -> UnifiedError {
        let error_info = ErrorInfo::new(Caller::Library(false, None));
        UnifiedError::AisError(error_info, error)
    }
}

impl UnifiedError {
    /// Creates a new `UnifiedError` instance from a `LoggerError`.
    ///
    /// Parameters:
    /// - `error`: The `LoggerError` instance.
    ///
    /// Returns:
    /// - `UnifiedError`: A new instance of `UnifiedError` with the appropriate `ErrorInfo`.
    pub fn from_logger_error(error: LoggerError) -> Self {
        let error_info = ErrorInfo::new(Caller::Library(true, Some(String::from("Logger Lib"))));
        UnifiedError::LoggerError(error_info, error)
    }

    /// Creates a new `UnifiedError` instance from a `SystemError`.
    ///
    /// Parameters:
    /// - `error`: The `SystemError` instance.
    ///
    /// Returns:
    /// - `UnifiedError`: A new instance of `UnifiedError` with the appropriate `ErrorInfo`.
    pub fn from_system_error(error: SystemError) -> Self {
        let error_info = ErrorInfo::new(Caller::Library(true, Some(String::from("System Lib"))));
        UnifiedError::SystemError(error_info, error)
    }

    /// Creates a new `UnifiedError` instance from a `RecsError`.
    ///
    /// Parameters:
    /// - `error`: The `RecsError` instance.
    ///
    /// Returns:
    /// - `UnifiedError`: A new instance of `UnifiedError` with the appropriate `ErrorInfo`.
    pub fn from_recs_error(error: RecsError) -> Self {
        let error_info = ErrorInfo::new(Caller::Library(
            true,
            Some(String::from("Rust Encryption Code System Lib")),
        ));
        UnifiedError::RecsError(error_info, error)
    }

    /// Creates a new `UnifiedError` instance from a `GitError`.
    ///
    /// Parameters:
    /// - `error`: The `GitError` instance.
    ///
    /// Returns:
    /// - `UnifiedError`: A new instance of `UnifiedError` with the appropriate `ErrorInfo`.
    pub fn from_git_error(error: GitError) -> Self {
        let error_info = ErrorInfo::new(Caller::Library(false, None));
        UnifiedError::GitError(error_info, error)
    }

    /// Creates a new `UnifiedError` instance from an `AisError`.
    ///
    /// Parameters:
    /// - `error`: The `AisError` instance.
    ///
    /// Returns:
    /// - `UnifiedError`: A new instance of `UnifiedError` with the appropriate `ErrorInfo`.
    pub fn from_ais_error(error: AisError) -> Self {
        let error_info = ErrorInfo::new(Caller::Library(false, None));
        UnifiedError::AisError(error_info, error)
    }
}

/// Enum representing different types of Ais errors.
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
    /// Git command didn't run successfully error.
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
    /// When running the first run system.
    FirstRun(Option<String>),
}

impl AisError {
    /// Creates a new AisError with the specified description.
    pub fn new(description: impl Into<String>) -> AisError {
        AisError::SystemError(Some(description.into()))
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
            | AisError::EncryptionNotReady(desc)
            | AisError::FirstRun(desc) => {
                desc.as_deref().unwrap_or("An unspecified error occurred")
            }
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
    Utf8Error(Utf8Error),
    // /// Git warning.
    // Warning(GitWarning),
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
            GitError::GitNotInstalled => "Git is not installed",
        }
    }
}

impl fmt::Display for UnifiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnifiedError::LoggerError(info, error) => {
                write!(f, "{} Logger error: {}", info.severity, error)
            }
            UnifiedError::SystemError(info, error) => {
                write!(f, "{} System error: {}", info.severity, error)
            }
            UnifiedError::RecsError(info, error) => {
                write!(f, "{} RECS error: {}", info.severity, error)
            }
            UnifiedError::GitError(info, error) => {
                write!(f, "{} Git error: {}", info.severity, error.description())
            }
            UnifiedError::AisError(info, error) => {
                write!(f, "{} AIS error: {}", info.severity, error.description())
            }
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_str = match self {
            Severity::Fatal => "Fatal",
            Severity::NotFatal => "NotFatal", // While its non fatle by nature if we don't catch and remedy this It'll be here
            Severity::Warning => "Warning",
        };
        write!(f, "{}", severity_str)
    }
}

impl fmt::Display for TimestampType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimestampType::CurrentTime(dt) => write!(f, "Current Time: {}", dt),
            TimestampType::_Custom(dt) => write!(f, "Custom Timestamp: {}", dt),
        }
    }
}

impl fmt::Display for Caller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Caller::Impl(bool_val, opt_str) => {
                if let Some(caller_name) = opt_str {
                    write!(f, "Impl (Bool: {}, Name: {})", bool_val, caller_name)
                } else {
                    write!(f, "Impl (Bool: {})", bool_val)
                }
            }
            Caller::Function(bool_val, opt_str) => {
                if let Some(caller_name) = opt_str {
                    write!(f, "Function (Bool: {}, Name: {})", bool_val, caller_name)
                } else {
                    write!(f, "Function (Bool: {})", bool_val)
                }
            }
            Caller::Library(bool_val, opt_str) => {
                if let Some(caller_name) = opt_str {
                    write!(f, "Library (Bool: {}, Name: {})", bool_val, caller_name)
                } else {
                    write!(f, "Library (Bool: {})", bool_val)
                }
            }
        }
    }
}

impl fmt::Display for AisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::CommandFailed(_) => write!(f, "Git command failed"),
            GitError::IoError(_) => write!(f, "IO error"),
            GitError::Utf8Error(_) => write!(f, "UTF-8 error"),
            // GitError::Warning(_) => write!(f, "Git warning"),
            GitError::GitNotInstalled => write!(f, "Git is not installed"),
        }
    }
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error Info:\nTimestamp: {}\nCaller: {}\nSeverity: {}",
            self.timestamp, self.caller, self.severity
        )
    }
}
