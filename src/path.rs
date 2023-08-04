#[cfg(not(target_os = "windows"))]
mod unix;

#[cfg(not(target_os = "windows"))]
pub use crate::path::unix::expand_env_var;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use crate::path::windows::expand_env_var;

use std::path::{Path, PathBuf};

pub fn strip_home<P>(path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    if let Some(home) = dirs::home_dir() {
        let path = path.as_ref();
        if let std::result::Result::Ok(path) = path.strip_prefix(&home) {
            PathBuf::from("~").join(path)
        } else {
            path.to_path_buf()
        }
    } else {
        path.as_ref().to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_home() {
        let home = dirs::home_dir().unwrap();
        let path = home.join("test");
        assert_eq!(strip_home(path), PathBuf::from("~").join("test"));
    }
}
