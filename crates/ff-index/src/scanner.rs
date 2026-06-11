use crate::entry::FileEntry;
use crate::store::Index;
use ff_common::types::{FileType, GitStatus};
use ignore::WalkBuilder;
use std::collections::HashMap;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub respect_ignore: bool,
    pub include_hidden: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            respect_ignore: true,
            include_hidden: false,
        }
    }
}

struct GitStatusResult {
    in_git_repo: bool,
    statuses: HashMap<std::path::PathBuf, GitStatus>,
}

pub fn scan(root: &Path, options: &ScanOptions) -> Result<Index, std::io::Error> {
    let git_result = collect_git_status(root);

    let mut entries = Vec::new();
    let mut walker = WalkBuilder::new(root);

    if !options.respect_ignore {
        walker
            .ignore(false)
            .git_ignore(false)
            .git_global(false)
            .git_exclude(false);
    }

    if options.include_hidden {
        walker.hidden(false);
    }

    for entry in walker.build() {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!("failed to read entry: {e}");
                continue;
            }
        };

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("failed to read metadata for {:?}: {e}", entry.path());
                continue;
            }
        };

        let path = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path())
            .to_path_buf();

        if path.as_os_str().is_empty() {
            continue;
        }

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .and_then(|d| i64::try_from(d.as_nanos()).ok())
            .unwrap_or(0);

        let atime = metadata
            .accessed()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .and_then(|d| i64::try_from(d.as_nanos()).ok())
            .unwrap_or(0);

        let ctime = {
            let sec = metadata.ctime();
            let nsec = metadata.ctime_nsec();
            let nanos = (sec as i128) * 1_000_000_000 + (nsec as i128);
            i64::try_from(nanos).unwrap_or(0)
        };

        let git_status = if git_result.in_git_repo {
            git_result
                .statuses
                .get(&path)
                .copied()
                .unwrap_or(GitStatus::Unmodified)
        } else {
            GitStatus::NotInGit
        };

        entries.push(FileEntry {
            path,
            size: metadata.len(),
            mtime,
            atime,
            ctime,
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            inode: metadata.ino(),
            device: metadata.dev(),
            link_count: metadata.nlink(),
            file_type: FileType::from_mode(metadata.mode()),
            git_status,
        });
    }

    let mut index = Index::new(root.to_path_buf());
    index.set_entries(entries);
    Ok(index)
}

fn collect_git_status(root: &Path) -> GitStatusResult {
    let output = match std::process::Command::new("git")
        .args(["status", "--porcelain=v1", "-uall"])
        .current_dir(root)
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            return GitStatusResult {
                in_git_repo: false,
                statuses: HashMap::new(),
            };
        }
    };

    if !output.status.success() {
        return GitStatusResult {
            in_git_repo: false,
            statuses: HashMap::new(),
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut statuses = HashMap::new();

    for line in stdout.lines() {
        if line.len() < 4 {
            continue;
        }

        let index_byte = line.as_bytes()[0];
        let worktree_byte = line.as_bytes()[1];

        let path_str = if (index_byte == b'R' || index_byte == b'C') && line.contains(" -> ") {
            line[3..].split(" -> ").last().unwrap_or(&line[3..])
        } else {
            &line[3..]
        };

        let path = std::path::PathBuf::from(path_str);
        let status = GitStatus::from_porcelain(index_byte, worktree_byte);
        statuses.insert(path, status);
    }

    GitStatusResult {
        in_git_repo: true,
        statuses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn scan_empty_directory() {
        let dir = TempDir::new().unwrap();
        let index = scan(dir.path(), &ScanOptions::default()).unwrap();
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn scan_single_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.txt"), "hello").unwrap();

        let index = scan(dir.path(), &ScanOptions::default()).unwrap();
        assert_eq!(index.len(), 1);

        let entry = index.get_by_path(Path::new("test.txt")).unwrap();
        assert_eq!(entry.size, 5);
        assert_eq!(entry.file_type, FileType::RegularFile);
    }

    #[test]
    fn scan_nested_directory() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub/file.txt"), "content").unwrap();

        let index = scan(dir.path(), &ScanOptions::default()).unwrap();
        assert!(index.len() >= 2);

        let entry = index.get_by_path(Path::new("sub/file.txt"));
        assert!(entry.is_some());
    }

    #[test]
    fn scan_respects_gitignore() {
        let dir = TempDir::new().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        fs::write(dir.path().join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(dir.path().join("included.txt"), "a").unwrap();
        fs::write(dir.path().join("ignored.txt"), "b").unwrap();

        let index = scan(dir.path(), &ScanOptions::default()).unwrap();
        assert!(index.get_by_path(Path::new("included.txt")).is_some());
        assert!(index.get_by_path(Path::new("ignored.txt")).is_none());
    }

    #[test]
    fn scan_no_ignore_option() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(dir.path().join("ignored.txt"), "b").unwrap();

        let options = ScanOptions {
            respect_ignore: false,
            include_hidden: true,
        };
        let index = scan(dir.path(), &options).unwrap();
        assert!(index.get_by_path(Path::new("ignored.txt")).is_some());
    }

    #[test]
    fn scan_metadata_collected() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("test.txt"), "hello world").unwrap();

        let index = scan(dir.path(), &ScanOptions::default()).unwrap();
        let entry = index.get_by_path(Path::new("test.txt")).unwrap();

        assert_eq!(entry.size, 11);
        assert!(entry.mtime > 0);
        assert!(entry.inode > 0);
        assert_eq!(entry.file_type, FileType::RegularFile);
    }
}
