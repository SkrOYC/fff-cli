use std::path::PathBuf;

#[must_use]
pub fn socket_path(root: &std::path::Path) -> PathBuf {
    let hash = hash_hex(root.to_string_lossy().as_ref());
    let short_hash = &hash[..16];
    runtime_dir().join(format!("{short_hash}.sock"))
}

#[must_use]
pub fn pid_path(root: &std::path::Path) -> PathBuf {
    let hash = hash_hex(root.to_string_lossy().as_ref());
    let short_hash = &hash[..16];
    runtime_dir().join(format!("{short_hash}.pid"))
}

#[must_use]
pub fn log_path(root: &std::path::Path) -> PathBuf {
    let hash = hash_hex(root.to_string_lossy().as_ref());
    let short_hash = &hash[..16];
    log_dir().join(format!("{short_hash}.log"))
}

#[must_use]
fn runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("ff");
    }

    if let Ok(uid) = std::env::var("UID") {
        return PathBuf::from(format!("/tmp/fffd-{uid}"));
    }

    if let Ok(user) = std::env::var("USER") {
        return PathBuf::from(format!("/tmp/fffd-{user}"));
    }

    PathBuf::from("/tmp/fffd")
}

#[must_use]
fn log_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(dir).join("ff").join("logs");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("ff")
            .join("logs");
    }

    PathBuf::from("/tmp/ff-logs")
}

fn hash_hex(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let h1 = hasher.finish();

    let mut hasher2 = DefaultHasher::new();
    h1.hash(&mut hasher2);
    input.hash(&mut hasher2);
    let h2 = hasher2.finish();

    format!("{h1:016x}{h2:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn socket_path_returns_sock_extension() {
        let path = socket_path(Path::new("/home/user/project"));
        assert!(path.to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn pid_path_returns_pid_extension() {
        let path = pid_path(Path::new("/home/user/project"));
        assert!(path.to_string_lossy().ends_with(".pid"));
    }

    #[test]
    fn log_path_returns_log_extension() {
        let path = log_path(Path::new("/home/user/project"));
        assert!(path.to_string_lossy().ends_with(".log"));
    }

    #[test]
    fn same_root_produces_same_paths() {
        let root = Path::new("/home/user/project");
        assert_eq!(socket_path(root), socket_path(root));
        assert_eq!(pid_path(root), pid_path(root));
        assert_eq!(log_path(root), log_path(root));
    }

    #[test]
    fn different_roots_produce_different_paths() {
        let root1 = Path::new("/home/user/project1");
        let root2 = Path::new("/home/user/project2");
        assert_ne!(socket_path(root1), socket_path(root2));
    }
}
