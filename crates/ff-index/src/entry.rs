use ff_common::types::{FileType, GitStatus};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: i64,
    pub atime: i64,
    pub ctime: i64,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub inode: u64,
    pub device: u64,
    pub link_count: u64,
    pub file_type: FileType,
    pub git_status: GitStatus,
}

impl FileEntry {
    #[must_use]
    pub fn depth(&self) -> usize {
        self.path.components().count()
    }

    #[must_use]
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name().and_then(|n| n.to_str())
    }

    #[must_use]
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }
}
