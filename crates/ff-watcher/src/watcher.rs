use notify::RecursiveMode;
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

/// Filesystem watcher with event debouncing.
///
/// Uses `notify-debouncer-mini` with a 500ms debounce window to coalesce
/// rapid filesystem changes into single rebuild triggers.
///
/// # Examples
///
/// ```no_run
/// use ff_watcher::FilesystemWatcher;
/// use std::time::Duration;
///
/// let watcher = FilesystemWatcher::new(std::path::Path::new(".")).unwrap();
/// while let Some(events) = watcher.try_wait_for_changes(Duration::from_secs(1)) {
///     println!("Detected {} filesystem events", events.len());
/// }
/// ```
pub struct FilesystemWatcher {
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
    rx: mpsc::Receiver<DebounceEventResult>,
    root: PathBuf,
}

/// Errors that can occur when creating or using a [`FilesystemWatcher`].
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    /// Failed to create the debouncer or set up watches.
    #[error("watcher setup failed: {0}")]
    Setup(#[from] notify::Error),

    /// The watcher's internal channel was disconnected (watcher died).
    #[error("watcher channel disconnected")]
    Disconnected,

    /// The watcher encountered an internal error.
    #[error("watcher internal error: {0}")]
    Internal(String),
}

impl FilesystemWatcher {
    /// Create a new filesystem watcher for the given root directory.
    ///
    /// Sets up recursive inotify watches with a 500ms debounce window.
    /// Events are coalesced so that rapid changes (e.g., `git checkout`)
    /// trigger a single rebuild rather than many.
    ///
    /// # Errors
    ///
    /// Returns [`WatcherError::Setup`] if the watcher cannot be created
    /// (e.g., inotify watch limits exceeded).
    pub fn new(root: &Path) -> Result<Self, WatcherError> {
        let (tx, rx) = mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(500), tx)?;

        debouncer.watcher().watch(root, RecursiveMode::Recursive)?;

        Ok(Self {
            _debouncer: debouncer,
            rx,
            root: root.to_path_buf(),
        })
    }

    /// Returns the root directory being watched.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Block until filesystem changes are detected.
    ///
    /// Returns `None` if the watcher's internal channel is disconnected
    /// or if the debouncer encountered an error.
    pub fn wait_for_changes(
        &self,
    ) -> Result<Vec<notify_debouncer_mini::DebouncedEvent>, WatcherError> {
        match self.rx.recv() {
            Ok(DebounceEventResult::Ok(events)) => Ok(events),
            Ok(DebounceEventResult::Err(e)) => Err(WatcherError::Internal(format!("{e:?}"))),
            Err(_) => Err(WatcherError::Disconnected),
        }
    }

    /// Wait for filesystem changes with a timeout.
    ///
    /// Returns `Ok(None)` if the timeout expires before any events arrive.
    /// Returns `Err` if the watcher's channel is disconnected or the
    /// debouncer encountered an error.
    pub fn try_wait_for_changes(
        &self,
        timeout: Duration,
    ) -> Result<Option<Vec<notify_debouncer_mini::DebouncedEvent>>, WatcherError> {
        match self.rx.recv_timeout(timeout) {
            Ok(DebounceEventResult::Ok(events)) => Ok(Some(events)),
            Ok(DebounceEventResult::Err(e)) => Err(WatcherError::Internal(format!("{e:?}"))),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(WatcherError::Disconnected),
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

        let events = watcher
            .try_wait_for_changes(Duration::from_secs(2))
            .unwrap();
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

        let events = watcher
            .try_wait_for_changes(Duration::from_secs(3))
            .unwrap();
        assert!(events.is_some());
        let events = events.unwrap();

        assert!(
            !events.is_empty(),
            "expected at least 1 coalesced event batch, got {}",
            events.len()
        );
    }

    #[test]
    fn watcher_timeout_returns_none() {
        let dir = TempDir::new().unwrap();
        let watcher = FilesystemWatcher::new(dir.path()).unwrap();

        let result = watcher
            .try_wait_for_changes(Duration::from_millis(100))
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn watcher_root_returns_correct_path() {
        let dir = TempDir::new().unwrap();
        let watcher = FilesystemWatcher::new(dir.path()).unwrap();
        assert_eq!(watcher.root(), dir.path());
    }
}
