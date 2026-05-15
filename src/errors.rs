//! Unified error types, error codes, exit codes, and structured error output
//!
//! CLI error contract: every error must have:
//! - message (human-readable)
//! - error_code (stable string)
//! - exit_code (u8)
//! - trace_id (optional)

use serde::Serialize;
use thiserror::Error;

// Exit codes (i32, compatible with Unix conventions)
#[allow(dead_code)]
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_GENERAL: i32 = 1;       // general error
pub const EXIT_USAGE: i32 = 2;        // argument/usage error
pub const EXIT_WORKSPACE: i32 = 3;     // workspace/path error
pub const EXIT_PROTOCOL: i32 = 4;      // windlocal protocol error
pub const EXIT_IO: i32 = 5;            // IO/permission error
pub const EXIT_PLATFORM: i32 = 6;      // platform/environment error
pub const EXIT_NETWORK: i32 = 7;       // network/version check error

// Error code strings (stable, machine-readable)

// Workspace / path / file operations: 100-199
pub const ERR_PATH_TRAVERSAL: &str = "PATH_TRAVERSAL";
pub const ERR_SYMLINK_NOT_SUPPORTED: &str = "SYMLINK_NOT_SUPPORTED";
pub const ERR_PATH_OUTSIDE_WORKSPACE: &str = "PATH_OUTSIDE_WORKSPACE";
pub const ERR_PATH_NOT_FOUND: &str = "PATH_NOT_FOUND";
pub const ERR_PATH_EXISTS: &str = "PATH_EXISTS";
pub const ERR_FILE_EXISTS: &str = "FILE_EXISTS";
pub const ERR_PATH_IS_DIR: &str = "PATH_IS_DIR";
pub const ERR_PATH_IS_NOT_DIR: &str = "PATH_IS_NOT_DIR";
pub const ERR_ATOMIC_RENAME_FAILED: &str = "ATOMIC_RENAME_FAILED";
pub const ERR_FILE_TOO_LARGE: &str = "FILE_TOO_LARGE";
pub const ERR_PERMISSION_DENIED: &str = "PERMISSION_DENIED";
pub const ERR_DISK_FULL: &str = "DISK_FULL";
pub const ERR_NO_ACTIVE_WORKSPACE: &str = "NO_ACTIVE_WORKSPACE";
pub const ERR_DIR_NOT_EMPTY: &str = "DIR_NOT_EMPTY";
pub const ERR_GLOB_NOT_ALLOWED: &str = "GLOB_NOT_ALLOWED";

// windlocal / URI / action schema: 200-299
pub const ERR_INVALID_SCHEME: &str = "INVALID_SCHEME";
pub const ERR_INVALID_ACTION_TYPE: &str = "INVALID_ACTION_TYPE";
pub const ERR_INVALID_PAGE_KIND: &str = "INVALID_PAGE_KIND";
pub const ERR_INVALID_COMMAND_ID: &str = "INVALID_COMMAND_ID";
pub const ERR_UNKNOWN_PARAM: &str = "UNKNOWN_PARAM";
pub const ERR_MISSING_PARAM: &str = "MISSING_PARAM";
pub const ERR_ACTION_BLOCKED: &str = "ACTION_BLOCKED";
pub const ERR_HIGH_RISK_REQUIRED_FORCE: &str = "HIGH_RISK_REQUIRED_FORCE";

// Platform / OS / environment: 300-399
pub const ERR_PLATFORM_UNSUPPORTED: &str = "PLATFORM_UNSUPPORTED";
pub const ERR_CONFIG_PATH_UNWRITABLE: &str = "CONFIG_PATH_UNWRITABLE";
pub const ERR_INIT_FAILED: &str = "INIT_FAILED";

// Upgrade / network / release: 400-499
pub const ERR_NETWORK_FAILED: &str = "NETWORK_FAILED";
pub const ERR_UPGRADE_SOURCE_UNREACHABLE: &str = "UPGRADE_SOURCE_UNREACHABLE";
pub const ERR_UPGRADE_RESPONSE_INVALID: &str = "UPGRADE_RESPONSE_INVALID";

// Generic fallback errors
pub const ERR_GENERAL: &str = "GENERAL_ERROR";
pub const ERR_IO: &str = "IO_ERROR";

// =============================================================================
// Error types
// =============================================================================

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum WindError {
    #[error("path traversal attempt detected")]
    PathTraversal,

    #[error("symlink paths are not supported in P0")]
    SymlinkNotSupported,

    #[error("path '{0}' is outside workspace root")]
    PathOutsideWorkspace(String),

    #[error("path not found: {0}")]
    PathNotFound(String),

    #[error("path already exists: {0}")]
    PathExists(String),

    #[error("file already exists: {0}")]
    FileExists(String),

    #[error("path is a directory: {0}")]
    PathIsDir(String),

    #[error("path is not a directory: {0}")]
    PathIsNotDir(String),

    #[error("atomic rename failed: {0}")]
    AtomicRenameFailed(String),

    #[error("file exceeds size limit ({limit} bytes): {path}")]
    FileTooLarge { limit: u64, path: String },

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("disk full")]
    DiskFull,

    #[error("no active workspace; run 'wind init' first")]
    NoActiveWorkspace,

    #[error("directory not empty: {0}")]
    DirNotEmpty(String),

    #[error("glob/wildcard is not allowed")]
    GlobNotAllowed,

    #[error("invalid URI scheme: {0}")]
    InvalidScheme(String),

    #[error("invalid action type")]
    InvalidActionType,

    #[error("invalid page kind: {0}")]
    InvalidPageKind(String),

    #[error("invalid command id: {0}")]
    InvalidCommandId(String),

    #[error("unknown parameter: {0}")]
    UnknownParam(String),

    #[error("missing required parameter: {0}")]
    MissingParam(String),

    #[error("action is blocked in P0: {0}")]
    ActionBlocked(String),

    #[error("platform unsupported: {0}")]
    PlatformUnsupported(String),

    #[error("config path is not writable: {0}")]
    ConfigPathUnwritable(String),

    #[error("workspace init failed: {0}")]
    InitFailed(String),

    #[error("network request failed: {0}")]
    NetworkFailed(String),

    #[error("upgrade source unreachable")]
    UpgradeSourceUnreachable,

    #[error("invalid upgrade response format")]
    UpgradeResponseInvalid,

    #[error("usage error: {0}")]
    Usage(String),

    #[error("HIGH_RISK_OPERATION: '{0}' requires --force flag")]
    HighRiskRequiresForce(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl WindError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::PathTraversal
            | Self::SymlinkNotSupported
            | Self::PathOutsideWorkspace(_)
            | Self::PathNotFound(_)
            | Self::PathExists(_)
            | Self::PathIsDir(_)
            | Self::PathIsNotDir(_)
            | Self::DirNotEmpty(_)
            | Self::NoActiveWorkspace
            | Self::FileExists(_) => EXIT_WORKSPACE,

            Self::InvalidScheme(_)
            | Self::InvalidActionType
            | Self::InvalidPageKind(_)
            | Self::InvalidCommandId(_)
            | Self::UnknownParam(_)
            | Self::MissingParam(_)
            | Self::ActionBlocked(_) => EXIT_PROTOCOL,

            Self::AtomicRenameFailed(_)
            | Self::FileTooLarge { .. }
            | Self::PermissionDenied(_)
            | Self::DiskFull | Self::GlobNotAllowed => EXIT_IO,

            Self::PlatformUnsupported(_)
            | Self::ConfigPathUnwritable(_)
            | Self::InitFailed(_) => EXIT_PLATFORM,

            Self::NetworkFailed(_)
            | Self::UpgradeSourceUnreachable
            | Self::UpgradeResponseInvalid => EXIT_NETWORK,

            Self::Usage(_) | Self::HighRiskRequiresForce(_) => EXIT_USAGE,
            Self::Io(_) => EXIT_IO,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::PathTraversal => ERR_PATH_TRAVERSAL,
            Self::SymlinkNotSupported => ERR_SYMLINK_NOT_SUPPORTED,
            Self::PathOutsideWorkspace(_) => ERR_PATH_OUTSIDE_WORKSPACE,
            Self::PathNotFound(_) => ERR_PATH_NOT_FOUND,
            Self::PathExists(_) => ERR_PATH_EXISTS,
            Self::FileExists(_) => ERR_FILE_EXISTS,
            Self::PathIsDir(_) => ERR_PATH_IS_DIR,
            Self::PathIsNotDir(_) => ERR_PATH_IS_NOT_DIR,
            Self::AtomicRenameFailed(_) => ERR_ATOMIC_RENAME_FAILED,
            Self::FileTooLarge { .. } => ERR_FILE_TOO_LARGE,
            Self::PermissionDenied(_) => ERR_PERMISSION_DENIED,
            Self::DiskFull => ERR_DISK_FULL,
            Self::NoActiveWorkspace => ERR_NO_ACTIVE_WORKSPACE,
            Self::DirNotEmpty(_) => ERR_DIR_NOT_EMPTY,
            Self::GlobNotAllowed => ERR_GLOB_NOT_ALLOWED,
            Self::InvalidScheme(_) => ERR_INVALID_SCHEME,
            Self::InvalidActionType => ERR_INVALID_ACTION_TYPE,
            Self::InvalidPageKind(_) => ERR_INVALID_PAGE_KIND,
            Self::InvalidCommandId(_) => ERR_INVALID_COMMAND_ID,
            Self::UnknownParam(_) => ERR_UNKNOWN_PARAM,
            Self::MissingParam(_) => ERR_MISSING_PARAM,
            Self::ActionBlocked(_) => ERR_ACTION_BLOCKED,
            Self::HighRiskRequiresForce(_) => ERR_HIGH_RISK_REQUIRED_FORCE,
            Self::PlatformUnsupported(_) => ERR_PLATFORM_UNSUPPORTED,
            Self::ConfigPathUnwritable(_) => ERR_CONFIG_PATH_UNWRITABLE,
            Self::InitFailed(_) => ERR_INIT_FAILED,
            Self::NetworkFailed(_) => ERR_NETWORK_FAILED,
            Self::UpgradeSourceUnreachable => ERR_UPGRADE_SOURCE_UNREACHABLE,
            Self::UpgradeResponseInvalid => ERR_UPGRADE_RESPONSE_INVALID,
            Self::Usage(_) => ERR_GENERAL,
            Self::Io(_) => ERR_IO,
        }
    }
}

// =============================================================================
// Structured error output
// =============================================================================

#[derive(Serialize, Debug)]
pub struct ErrorOutput {
    pub ok: bool,
    pub error: ErrorDetail,
}

#[derive(Serialize, Debug)]
pub struct ErrorDetail {
    #[serde(rename = "error_code")]
    pub code: String,
    #[serde(rename = "exitCode")]
    pub exit_code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl ErrorOutput {
    #[allow(dead_code)]
    pub fn new(err: &WindError, trace_id: Option<String>) -> Self {
        Self {
            ok: false,
            error: ErrorDetail {
                code: err.code().to_string(),
                exit_code: err.exit_code(),
                message: err.to_string(),
                trace_id,
            },
        }
    }
}

/// Print structured JSON error and exit
pub fn exit_with_error(err: &anyhow::Error, json_mode: bool) -> ! {
    let exit_code = if let Some(we) = err.downcast_ref::<WindError>() {
        we.exit_code()
    } else {
        EXIT_GENERAL
    };

    let code = if let Some(we) = err.downcast_ref::<WindError>() {
        we.code().to_string()
    } else {
        ERR_GENERAL.to_string()
    };

    let message = err.to_string();

    if json_mode {
        let output = ErrorOutput {
            ok: false,
            error: ErrorDetail {
                code,
                exit_code,
                message,
                trace_id: Some(uuid_simple()),
            },
        };
        eprintln!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        eprintln!("error: {}", err);
    }
    std::process::exit(exit_code);
}

// Minimal UUID v4 for trace_id (no external dependency)
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", now)
}
