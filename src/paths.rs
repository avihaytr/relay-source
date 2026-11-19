// SPDX-License-Identifier: MPL-2.0
//! Writable per-user locations for installed Linux builds.
use std::{path::{Path, PathBuf}, io::Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

fn base(xdg: Option<PathBuf>, home: &Path, fallback: &str) -> PathBuf {
    xdg.filter(|path| path.is_absolute()).unwrap_or_else(|| home.join(fallback)).join("relay")
}
fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).filter(|path| path.is_absolute())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}
pub fn settings_file() -> PathBuf {
    base(std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from), &home(), ".config").join("settings.json")
}
pub fn captures_dir() -> PathBuf {
    base(std::env::var_os("XDG_DATA_HOME").map(PathBuf::from), &home(), ".local/share").join("captures")
}
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("Settings directory is missing"))?;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(parent)?;
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let temporary = parent.join(format!(".settings-{}-{nonce}.new", std::process::id()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)
    })();
    if result.is_err() { let _ = std::fs::remove_file(temporary); }
    result
}
pub fn save_settings(bytes: &[u8]) -> std::io::Result<()> { write_atomic(&settings_file(), bytes) }

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn xdg_paths_require_absolute_values_and_fall_back_to_home() {
        let home = Path::new("/test-home");
        assert_eq!(base(Some("/config".into()), home, ".config"), Path::new("/config/relay"));
        assert_eq!(base(Some("relative".into()), home, ".config"), Path::new("/test-home/.config/relay"));
        assert_eq!(base(None, home, ".local/share"), Path::new("/test-home/.local/share/relay"));
    }
    #[test]
    fn settings_create_private_parent_and_replace_complete_contents() {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let directory = std::env::temp_dir().join(format!("relay-settings-test-{}-{nonce}", std::process::id()));
        let path = directory.join("nested/settings.json");
        write_atomic(&path, br#"{"settings":"first"}"#).unwrap();
        write_atomic(&path, br#"{"settings":"second"}"#).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), br#"{"settings":"second"}"#);
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(std::fs::read_dir(path.parent().unwrap()).unwrap().count(), 1);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
