use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FileType {
    RegularFile = 0,
    Directory = 1,
    Symlink = 2,
    Socket = 3,
    BlockDevice = 4,
    CharacterDevice = 5,
    Fifo = 6,
    Unknown = 7,
}

impl FileType {
    #[must_use]
    pub fn from_mode(mode: u32) -> Self {
        let ft = mode & 0o170000;
        match ft {
            0o100000 => Self::RegularFile,
            0o040000 => Self::Directory,
            0o120000 => Self::Symlink,
            0o140000 => Self::Socket,
            0o060000 => Self::BlockDevice,
            0o020000 => Self::CharacterDevice,
            0o010000 => Self::Fifo,
            _ => Self::Unknown,
        }
    }

    #[must_use]
    pub fn short_name(self) -> &'static str {
        match self {
            Self::RegularFile => "f",
            Self::Directory => "d",
            Self::Symlink => "l",
            Self::Socket => "s",
            Self::BlockDevice => "b",
            Self::CharacterDevice => "c",
            Self::Fifo => "p",
            Self::Unknown => "?",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum GitStatus {
    Untracked = 0,
    Unmodified = 1,
    Modified = 2,
    Added = 3,
    Deleted = 4,
    Renamed = 5,
    Copied = 6,
    UpdatedButUnmerged = 7,
    NotInGit = 255,
}

impl GitStatus {
    #[must_use]
    pub fn from_porcelain(index: u8, worktree: u8) -> Self {
        match (index, worktree) {
            (b'?', b'?') => Self::Untracked,
            (b' ', b' ') => Self::Unmodified,
            (b' ', b'M') | (b'M', _) => Self::Modified,
            (b'A', _) => Self::Added,
            (b'D', _) | (b' ', b'D') => Self::Deleted,
            (b'R', _) => Self::Renamed,
            (b'C', _) => Self::Copied,
            (b'U', _) | (_, b'U') => Self::UpdatedButUnmerged,
            _ => Self::NotInGit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CaseMode {
    #[default]
    Smart,
    Sensitive,
    Insensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PatternMode {
    #[default]
    Regex,
    Glob,
    FixedString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExecMode {
    #[default]
    PerMatch,
    Batch,
    PerMatchDir,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_type_from_mode_regular_file() {
        assert_eq!(FileType::from_mode(0o100644), FileType::RegularFile);
    }

    #[test]
    fn file_type_from_mode_directory() {
        assert_eq!(FileType::from_mode(0o040755), FileType::Directory);
    }

    #[test]
    fn file_type_from_mode_symlink() {
        assert_eq!(FileType::from_mode(0o120777), FileType::Symlink);
    }

    #[test]
    fn git_status_from_porcelain_untracked() {
        assert_eq!(GitStatus::from_porcelain(b'?', b'?'), GitStatus::Untracked);
    }

    #[test]
    fn git_status_from_porcelain_modified() {
        assert_eq!(GitStatus::from_porcelain(b' ', b'M'), GitStatus::Modified);
        assert_eq!(GitStatus::from_porcelain(b'M', b' '), GitStatus::Modified);
    }

    #[test]
    fn git_status_from_porcelain_added() {
        assert_eq!(GitStatus::from_porcelain(b'A', b' '), GitStatus::Added);
    }

    #[test]
    fn git_status_from_porcelain_deleted() {
        assert_eq!(GitStatus::from_porcelain(b' ', b'D'), GitStatus::Deleted);
        assert_eq!(GitStatus::from_porcelain(b'D', b' '), GitStatus::Deleted);
    }

    #[test]
    fn case_mode_default_is_smart() {
        assert_eq!(CaseMode::default(), CaseMode::Smart);
    }

    #[test]
    fn pattern_mode_default_is_regex() {
        assert_eq!(PatternMode::default(), PatternMode::Regex);
    }
}
