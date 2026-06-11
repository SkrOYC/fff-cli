use thiserror::Error;

#[derive(Debug, Error)]
pub enum FfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("index error: {0}")]
    Index(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("daemon not running")]
    DaemonNotRunning,

    #[error("daemon already running (PID {0})")]
    DaemonAlreadyRunning(u32),

    #[error("query timeout after {0}s")]
    QueryTimeout(u64),

    #[error("invalid pattern: {0}")]
    InvalidPattern(String),

    #[error("path not found: {0}")]
    PathNotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, FfError>;
