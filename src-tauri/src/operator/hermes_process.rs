use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn workspace_root() -> PathBuf {
    if let Ok(root) = env::var("ACP_UI_WORKSPACE_ROOT") {
        let root = PathBuf::from(root);
        if is_d_drive_path(&root) {
            return root;
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("D:/dingsun/acp-ui"));

    if is_d_drive_path(&root) {
        root
    } else {
        PathBuf::from("D:/dingsun/acp-ui")
    }
}

pub fn hermes_home_dir() -> PathBuf {
    if let Ok(home) = env::var("HERMES_HOME") {
        let home = PathBuf::from(home);
        if is_d_drive_path(&home) {
            return home;
        }
    }

    workspace_root().join("hermes")
}

pub fn apply_d_drive_hermes_env(command: &mut Command) {
    let root = workspace_root();
    let home = hermes_home_dir();
    let data = home.join("data");
    let config = home.join("config");
    let cache = home.join("cache");
    let appdata = data.join("appdata");
    let local_appdata = data.join("localappdata");

    let _ = fs::create_dir_all(&home);
    let _ = fs::create_dir_all(&data);
    let _ = fs::create_dir_all(&config);
    let _ = fs::create_dir_all(&cache);
    let _ = fs::create_dir_all(&appdata);
    let _ = fs::create_dir_all(&local_appdata);

    command
        .current_dir(root)
        .env("HERMES_HOME", &home)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("APPDATA", appdata)
        .env("LOCALAPPDATA", local_appdata)
        .env("XDG_DATA_HOME", data.join("xdg"))
        .env("XDG_CONFIG_HOME", &config)
        .env("XDG_CACHE_HOME", &cache);
}

pub fn d_drive_path_from_env(key: &str) -> Option<PathBuf> {
    env::var_os(key)
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .filter(|path| is_d_drive_path(path))
}

pub fn d_drive_path_override(key: &str) -> Result<Option<PathBuf>, String> {
    let Some(value) = env::var_os(key) else {
        return Ok(None);
    };

    validate_d_drive_path_override(key, PathBuf::from(value)).map(Some)
}

fn validate_d_drive_path_override(key: &str, path: PathBuf) -> Result<PathBuf, String> {
    if path.as_os_str().is_empty() || !path.is_absolute() || !is_d_drive_path(&path) {
        return Err(format!(
            "{} must be an absolute D: path, got '{}'",
            key,
            path.display()
        ));
    }
    Ok(path)
}

pub fn is_d_drive_path(path: &Path) -> bool {
    #[cfg(windows)]
    {
        let path = path.to_string_lossy().replace('/', "\\");
        let path = path
            .strip_prefix("\\\\?\\")
            .or_else(|| path.strip_prefix("\\\\.\\"))
            .unwrap_or(&path);
        let bytes = path.as_bytes();
        bytes.len() >= 2 && matches!(bytes[0], b'D' | b'd') && bytes[1] == b':'
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d_drive_guard_accepts_normal_and_extended_d_paths() {
        assert!(is_d_drive_path(Path::new("D:/games/project")));
        assert!(is_d_drive_path(Path::new("d:\\games\\project")));

        #[cfg(windows)]
        {
            assert!(is_d_drive_path(Path::new("\\\\?\\D:\\games\\project")));
            assert!(is_d_drive_path(Path::new("\\\\.\\d:\\games\\project")));
        }
    }

    #[test]
    fn d_drive_guard_rejects_other_drives_and_extended_unc_paths() {
        #[cfg(windows)]
        {
            assert!(!is_d_drive_path(Path::new("C:\\games\\project")));
            assert!(!is_d_drive_path(Path::new("\\\\?\\C:\\games\\project")));
            assert!(!is_d_drive_path(Path::new("\\\\?\\UNC\\server\\share")));
        }
    }

    #[test]
    fn d_drive_override_requires_an_absolute_d_path() {
        assert_eq!(
            validate_d_drive_path_override(
                "ACP_TEST_PATH",
                PathBuf::from("D:/runtime/config.json")
            )
            .expect("absolute D path"),
            PathBuf::from("D:/runtime/config.json")
        );

        #[cfg(windows)]
        {
            for invalid in ["", "relative/file", "D:relative", "C:/runtime/file"] {
                let error = validate_d_drive_path_override("ACP_TEST_PATH", PathBuf::from(invalid))
                    .expect_err("invalid override must fail");
                assert!(error.contains("ACP_TEST_PATH must be an absolute D: path"));
            }
        }
    }
}

pub fn executable_version(path: &Path) -> Option<String> {
    let mut command = Command::new(path);
    apply_d_drive_hermes_env(&mut command);
    let output = command.arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Some(stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return Some(stderr);
    }

    path.file_name()
        .map(|name| format!("{} (version output empty)", name.to_string_lossy()))
}
