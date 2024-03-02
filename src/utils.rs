const WINDOWS_PATH: &str = "\\";
const UNIX_PATH: &str = "/";

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn separator() -> &'static str {
    "/"
}

#[cfg(target_os = "windows")]
pub fn separator() -> &'static str {
    "\\"
}

pub fn safe_path(path: &str) -> String {
    let mut np = String::from(path);

    if cfg!(windows) {
        np = np.replace(UNIX_PATH, WINDOWS_PATH);
    }

    if cfg!(macos) || cfg!(linux) {
        np = np.replace(WINDOWS_PATH, UNIX_PATH);
    }

    np
}