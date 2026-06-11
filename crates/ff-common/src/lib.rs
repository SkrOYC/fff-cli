//! Shared types and utilities for ff.

pub mod config;
pub mod errors;
pub mod paths;
pub mod types;

pub use config::Config;
pub use errors::{FfError, Result};
pub use types::{CaseMode, ExecMode, FileType, GitStatus, PatternMode};
