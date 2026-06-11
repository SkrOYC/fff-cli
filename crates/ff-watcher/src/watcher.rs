use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

pub struct FilesystemWatcher {
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
    rx: mpsc::Receiver<DebounceEventResult>,
    root: PathBuf,
}

impl FilesystemWatcher {
    pub fn new(root: &Path) -> Result<Self, std::io::Error> {
        let (tx, rx) = mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx)
            .map_err(std::io::Error::other)?;

        debouncer
            .watcher()
            .watch(root, RecursiveMode::Recursive)
            .map_err(std::io::Error::other)?;

        Ok(Self {
            _debouncer: debouncer,
            rx,
            root: root.to_path_buf(),
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn wait_for_changes(&self) -> Option<Vec<notify_debouncer_mini::DebouncedEvent>> {
        match self.rx.recv() {
            Ok(DebounceEventResult::Ok(events)) => Some(events),
            Ok(DebounceEventResult::Err(_)) => None,
            Err(_) => None,
        }
    }

    pub fn try_wait_for_changes(
        &self,
        timeout: Duration,
    ) -> Option<Vec<notify_debouncer_mini::DebouncedEvent>> {
        match self.rx.recv_timeout(timeout) {
            Ok(DebounceEventResult::Ok(events)) => Some(events),
            Ok(DebounceEventResult::Err(_)) => None,
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn watcher_detects_file_creation() {
        let dir = TempDir::new().unwrap();
        let watcher = FilesystemWatcher::new(dir.path()).unwrap();

        fs::write(dir.path().join("test.txt"), "hello").unwrap();

        let events = watcher.try_wait_for_changes(Duration::from_secs(2));
        assert!(events.is_some());
        let events = events.unwrap();
        assert!(!events.is_empty());
    }

    #[test]
    fn watcher_coalesces_burst() {
        let dir = TempDir::new().unwrap();
        let watcher = FilesystemWatcher::new(dir.path()).unwrap();

        for i in 0..100 {
            fs::write(dir.path().join(format!("file_{i}.txt")), "x").unwrap();
        }

        let events = watcher.try_wait_for_changes(Duration::from_secs(3));
        assert!(events.is_some());
    }

    #[test]
    fn watcher_root_returns_correct_path() {
        let dir = TempDir::new().unwrap();
        let watcher = FilesystemWatcher::new(dir.path()).unwrap();
        assert_eq!(watcher.root(), dir.path());
    }
}
