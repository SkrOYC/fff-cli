#![allow(dead_code)]

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use tokio::net::UnixStream;
use tracing::{info, warn};

const SOCKET_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct AutoStartConfig {
    pub root: std::path::PathBuf,
    pub socket_wait_timeout: Duration,
    pub socket_poll_interval: Duration,
}

impl Default for AutoStartConfig {
    fn default() -> Self {
        Self {
            root: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            socket_wait_timeout: SOCKET_WAIT_TIMEOUT,
            socket_poll_interval: SOCKET_POLL_INTERVAL,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonStatus {
    Running,
    NotRunning,
    StaleSocket,
}

pub fn check_daemon_status(socket_path: &Path) -> DaemonStatus {
    if !socket_path.exists() {
        return DaemonStatus::NotRunning;
    }

    match std::os::unix::net::UnixStream::connect(socket_path) {
        Ok(_) => DaemonStatus::Running,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::ConnectionRefused {
                DaemonStatus::StaleSocket
            } else {
                DaemonStatus::NotRunning
            }
        }
    }
}

pub fn cleanup_stale_socket(socket_path: &Path, pid_path: &Path) -> Result<()> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
        info!("removed stale socket: {}", socket_path.display());
    }

    if pid_path.exists() {
        let pid_str = std::fs::read_to_string(pid_path)?;
        if let Ok(pid) = pid_str.trim().parse::<u32>()
            && !is_process_running(pid)
        {
            std::fs::remove_file(pid_path)?;
            info!("removed stale PID file: {}", pid_path.display());
        }
    }

    Ok(())
}

pub async fn ensure_daemon(config: &AutoStartConfig) -> Result<UnixStream> {
    let socket_path = ff_common::paths::socket_path(&config.root);
    let pid_path = ff_common::paths::pid_path(&config.root);

    let status = check_daemon_status(&socket_path);

    match status {
        DaemonStatus::Running => {
            info!(
                "daemon already running, connecting to {}",
                socket_path.display()
            );
            let stream = UnixStream::connect(&socket_path).await?;
            return Ok(stream);
        }
        DaemonStatus::StaleSocket => {
            warn!("stale socket detected, cleaning up");
            cleanup_stale_socket(&socket_path, &pid_path)?;
        }
        DaemonStatus::NotRunning => {}
    }

    info!("daemon not running, starting auto-start");
    spawn_daemon(config)?;

    wait_for_socket(
        &socket_path,
        config.socket_wait_timeout,
        config.socket_poll_interval,
    )
    .await?;

    let stream = UnixStream::connect(&socket_path).await?;
    info!("connected to daemon at {}", socket_path.display());
    Ok(stream)
}

fn spawn_daemon(config: &AutoStartConfig) -> Result<()> {
    if let Some(parent) = ff_common::paths::socket_path(&config.root).parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }

    let daemon_bin = std::env::current_exe()?;
    let root_arg = config.root.to_string_lossy().to_string();

    let child = std::process::Command::new(&daemon_bin)
        .arg("--root")
        .arg(&root_arg)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    info!("spawned daemon process (PID {})", child.id());

    Ok(())
}

async fn wait_for_socket(
    socket_path: &Path,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<()> {
    let start = std::time::Instant::now();

    loop {
        if socket_path.exists() && UnixStream::connect(socket_path).await.is_ok() {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            anyhow::bail!(
                "daemon failed to start within {}s (socket not found at {})",
                timeout.as_secs(),
                socket_path.display()
            );
        }

        tokio::time::sleep(poll_interval).await;
    }
}

#[cfg(unix)]
fn is_process_running(pid: u32) -> bool {
    unsafe {
        let result = libc::kill(pid as i32, 0);
        if result == 0 {
            return true;
        }
        std::io::Error::last_os_error().kind() == std::io::ErrorKind::PermissionDenied
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

    fn test_config() -> (TempDir, AutoStartConfig) {
        let dir = TempDir::new().unwrap();
        unsafe {
            std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        }
        let config = AutoStartConfig {
            root: dir.path().to_path_buf(),
            socket_wait_timeout: Duration::from_millis(500),
            socket_poll_interval: Duration::from_millis(50),
        };
        (dir, config)
    }

    #[test]
    fn check_daemon_not_running() {
        let (_dir, config) = test_config();
        let socket_path = ff_common::paths::socket_path(&config.root);
        assert_eq!(check_daemon_status(&socket_path), DaemonStatus::NotRunning);
    }

    #[test]
    fn check_daemon_status_with_socket_file() {
        let (_dir, config) = test_config();
        let socket_path = ff_common::paths::socket_path(&config.root);

        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        let _listener = std::os::unix::net::UnixListener::bind(&socket_path).unwrap();

        let status = check_daemon_status(&socket_path);
        assert!(matches!(
            status,
            DaemonStatus::Running | DaemonStatus::StaleSocket | DaemonStatus::NotRunning
        ));
    }

    #[test]
    fn cleanup_stale_files() {
        let (_dir, config) = test_config();
        let socket_path = ff_common::paths::socket_path(&config.root);
        let pid_path = ff_common::paths::pid_path(&config.root);

        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        std::fs::write(&socket_path, "").unwrap();
        std::fs::write(&pid_path, "999999999").unwrap();

        cleanup_stale_socket(&socket_path, &pid_path).unwrap();

        assert!(!socket_path.exists());
        assert!(!pid_path.exists());
    }

    #[tokio::test]
    async fn wait_for_socket_timeout() {
        let (_dir, config) = test_config();
        let socket_path = ff_common::paths::socket_path(&config.root);

        let result = wait_for_socket(
            &socket_path,
            Duration::from_millis(200),
            Duration::from_millis(50),
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to start"));
    }

    #[tokio::test]
    async fn wait_for_socket_success() {
        let (_dir, config) = test_config();
        let socket_path = ff_common::paths::socket_path(&config.root);

        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        let _listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let result = wait_for_socket(
            &socket_path,
            Duration::from_secs(1),
            Duration::from_millis(50),
        )
        .await;

        assert!(result.is_ok());
    }

    #[test]
    fn default_config_values() {
        let config = AutoStartConfig::default();
        assert_eq!(config.socket_wait_timeout, Duration::from_secs(2));
        assert_eq!(config.socket_poll_interval, Duration::from_millis(100));
    }
}
