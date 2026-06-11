use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tokio::net::UnixListener;
use tokio::sync::watch;
use tracing::{info, warn};

#[derive(Debug)]
#[allow(dead_code)]
pub struct DaemonLifecycle {
    root: PathBuf,
    socket_path: PathBuf,
    pid_path: PathBuf,
    idle_timeout: Duration,
    last_activity: Instant,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
}

#[allow(dead_code)]
impl DaemonLifecycle {
    pub fn new(root: PathBuf, idle_timeout: Duration) -> Self {
        let socket_path = ff_common::paths::socket_path(&root);
        let pid_path = ff_common::paths::pid_path(&root);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        Self {
            root,
            socket_path,
            pid_path,
            idle_timeout,
            last_activity: Instant::now(),
            shutdown_tx,
            shutdown_rx,
        }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    #[must_use]
    pub fn pid_path(&self) -> &Path {
        &self.pid_path
    }

    #[must_use]
    pub fn shutdown_receiver(&self) -> watch::Receiver<bool> {
        self.shutdown_rx.clone()
    }

    pub fn notify_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    #[must_use]
    pub fn is_idle_timed_out(&self) -> bool {
        self.last_activity.elapsed() >= self.idle_timeout
    }

    pub fn request_shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    pub fn check_and_clean_stale(&self) -> io::Result<()> {
        if self.pid_path.exists() {
            let pid_str = fs::read_to_string(&self.pid_path)?;
            let pid: u32 = pid_str.trim().parse().map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("invalid PID: {e}"))
            })?;

            if !is_process_running(pid) {
                warn!("stale PID file found (PID {pid} not running), cleaning up");
                self.cleanup_files()?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("daemon already running with PID {pid}"),
                ));
            }
        }

        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)?;
        }

        Ok(())
    }

    pub fn create_pid_file(&self) -> io::Result<()> {
        let pid = std::process::id();
        if let Some(parent) = self.pid_path.parent() {
            fs::create_dir_all(parent)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
            }
        }
        fs::write(&self.pid_path, pid.to_string())?;
        info!(
            "created PID file at {} (PID {pid})",
            self.pid_path.display()
        );
        Ok(())
    }

    pub fn create_socket(&self) -> io::Result<UnixListener> {
        if let Some(parent) = self.socket_path.parent() {
            fs::create_dir_all(parent)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
            }
        }

        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.socket_path, fs::Permissions::from_mode(0o600))?;
        }

        info!("created socket at {}", self.socket_path.display());
        Ok(listener)
    }

    pub fn cleanup_files(&self) -> io::Result<()> {
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)?;
            info!("removed socket at {}", self.socket_path.display());
        }
        if self.pid_path.exists() {
            fs::remove_file(&self.pid_path)?;
            info!("removed PID file at {}", self.pid_path.display());
        }
        Ok(())
    }
}

impl Drop for DaemonLifecycle {
    fn drop(&mut self) {
        if let Err(e) = self.cleanup_files() {
            warn!("failed to cleanup daemon files: {e}");
        }
    }
}

#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    unsafe {
        let result = libc::kill(pid as i32, 0);
        if result == 0 {
            return true;
        }
        io::Error::last_os_error().kind() == io::ErrorKind::PermissionDenied
    }
}

#[cfg(not(unix))]
fn is_process_running(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_root() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let root = dir.path().to_path_buf();
        unsafe {
            std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        }
        (dir, root)
    }

    #[test]
    fn new_lifecycle_creates_paths() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root.clone(), Duration::from_secs(300));

        assert_eq!(lifecycle.root(), root);
        assert!(lifecycle.socket_path().to_string_lossy().ends_with(".sock"));
        assert!(lifecycle.pid_path().to_string_lossy().ends_with(".pid"));
    }

    #[test]
    fn create_and_cleanup_pid_file() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root, Duration::from_secs(300));

        lifecycle.create_pid_file().unwrap();
        assert!(lifecycle.pid_path().exists());

        let pid_str = fs::read_to_string(lifecycle.pid_path()).unwrap();
        let pid: u32 = pid_str.trim().parse().unwrap();
        assert_eq!(pid, std::process::id());

        lifecycle.cleanup_files().unwrap();
        assert!(!lifecycle.pid_path().exists());
    }

    #[tokio::test]
    async fn create_socket_with_permissions() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root, Duration::from_secs(300));

        let _listener = lifecycle.create_socket().unwrap();
        assert!(lifecycle.socket_path().exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(lifecycle.socket_path()).unwrap();
            let mode = metadata.permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn stale_pid_detection() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root, Duration::from_secs(300));

        fs::create_dir_all(lifecycle.pid_path().parent().unwrap()).unwrap();
        fs::write(lifecycle.pid_path(), "999999999").unwrap();

        lifecycle.check_and_clean_stale().unwrap();
        assert!(!lifecycle.pid_path().exists());
    }

    #[test]
    fn running_pid_detected() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root, Duration::from_secs(300));

        let my_pid = std::process::id();
        fs::create_dir_all(lifecycle.pid_path().parent().unwrap()).unwrap();
        fs::write(lifecycle.pid_path(), my_pid.to_string()).unwrap();

        let result = lifecycle.check_and_clean_stale();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already running"));
    }

    #[test]
    fn idle_timeout_detection() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root, Duration::from_millis(100));

        assert!(!lifecycle.is_idle_timed_out());

        std::thread::sleep(Duration::from_millis(150));
        assert!(lifecycle.is_idle_timed_out());
    }

    #[test]
    fn notify_activity_resets_timeout() {
        let (_dir, root) = test_root();
        let mut lifecycle = DaemonLifecycle::new(root, Duration::from_millis(100));

        std::thread::sleep(Duration::from_millis(50));
        lifecycle.notify_activity();

        std::thread::sleep(Duration::from_millis(50));
        assert!(!lifecycle.is_idle_timed_out());
    }

    #[test]
    fn shutdown_signal() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root, Duration::from_secs(300));

        let mut rx = lifecycle.shutdown_receiver();
        assert!(!*rx.borrow());

        lifecycle.request_shutdown();
        rx.borrow_and_update();
        assert!(*rx.borrow());
    }

    #[tokio::test]
    async fn drop_cleans_up_files() {
        let (_dir, root) = test_root();
        let lifecycle = DaemonLifecycle::new(root, Duration::from_secs(300));

        lifecycle.create_pid_file().unwrap();
        let _listener = lifecycle.create_socket().unwrap();

        let pid_path = lifecycle.pid_path().to_path_buf();
        let socket_path = lifecycle.socket_path().to_path_buf();

        assert!(pid_path.exists());
        assert!(socket_path.exists());

        drop(lifecycle);

        assert!(!pid_path.exists());
        assert!(!socket_path.exists());
    }
}
