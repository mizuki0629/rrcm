//! Windows specific path utilities
//! This module is enabled only on windows
//! because Windows has different path format
//! (e.g. C:\Users\username\...)
//! and different path separator (e.g. C:\Users\username\foo\bar.txt)
use anyhow::Context as _;
use anyhow::{bail, Ok, Result};
use dirs::{config_dir, config_local_dir, desktop_dir, document_dir};
use std::path::PathBuf;

/// Get path of known folder
/// KNOWNFOLDERID is one of the following:
/// - FOLDERID_Desktop
/// - FOLDERID_Documents
/// - FOLDERID_LocalAppData
/// - FOLDERID_RoamingAppData
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
fn get_known_folder(s: &str) -> Result<String> {
    let mut path = PathBuf::new();
    match s {
        "FOLDERID_Desktop" => {
            path.push(desktop_dir().unwrap());
        }
        "FOLDERID_Documents" => {
            path.push(document_dir().unwrap());
        }
        "FOLDERID_LocalAppData" => {
            path.push(config_local_dir().unwrap());
        }
        "FOLDERID_RoamingAppData" => {
            path.push(config_dir().unwrap());
        }
        _ => bail!("invalid KNOWNFOLDERID: {}", s),
    }
    Ok(path.to_str().unwrap().to_string())
}

/// Check if the specified string is knownfolderid
/// KNOWNFOLDERID is one of the following:
/// - FOLDERID_Desktop
/// - FOLDERID_Documents
/// - FOLDERID_LocalAppData
/// - FOLDERID_RoamingAppData
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
fn is_known_folder_id(s: &str) -> bool {
    matches!(
        s,
        "FOLDERID_Desktop"
            | "FOLDERID_Documents"
            | "FOLDERID_LocalAppData"
            | "FOLDERID_RoamingAppData"
    )
}

/// Expand environment variable on windows
/// %VARNAME% format is supported
/// If VARNAME is KNOWNFOLDERID, expand to path of known folder
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
pub fn expand_env_var(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars();

    #[allow(clippy::while_let_on_iterator)]
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut varname = String::new();
            let mut is_end = false;
            while let Some(c) = chars.next() {
                if c == '%' {
                    is_end = true;
                    break;
                }
                varname.push(c);
            }
            if !is_end {
                bail!("invalid environment variable: {}", s);
            }
            if varname.is_empty() {
                bail!("invalid environment variable: {}", s);
            }
            if is_known_folder_id(&varname) {
                result.push_str(&get_known_folder(&varname)?);
                continue;
            }
            let value = std::env::var(&varname)
                .with_context(|| format!("env var {} not found", varname))?;
            result.push_str(&value);
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_var() {
        let home = dirs::home_dir().unwrap();
        let path = home.join("test");
        assert_eq!(
            expand_env_var("%USERPROFILE%\\test").unwrap(),
            path.to_str().unwrap()
        );
        assert_eq!(
            expand_env_var("%FOLDERID_Desktop%\\test").unwrap(),
            desktop_dir().unwrap().to_str().unwrap()
        );
        assert_eq!(
            expand_env_var("%FOLDERID_Documents%\\test").unwrap(),
            document_dir().unwrap().to_str().unwrap()
        );
        assert_eq!(
            expand_env_var("%FOLDERID_LocalAppData%\\test").unwrap(),
            config_local_dir().unwrap().to_str().unwrap()
        );
        assert_eq!(
            expand_env_var("%FOLDERID_RoamingAppData%\\test").unwrap(),
            config_dir().unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn test_is_known_folder_id() {
        assert!(is_known_folder_id("FOLDERID_Desktop"));
        assert!(is_known_folder_id("FOLDERID_Documents"));
        assert!(is_known_folder_id("FOLDERID_LocalAppData"));
        assert!(is_known_folder_id("FOLDERID_RoamingAppData"));
        assert!(!is_known_folder_id("FOLDERID_Desktop2"));
    }

    #[test]
    fn test_get_known_folder() {
        assert_eq!(
            get_known_folder("FOLDERID_Desktop").unwrap(),
            desktop_dir().unwrap().to_str().unwrap()
        );
        assert_eq!(
            get_known_folder("FOLDERID_Documents").unwrap(),
            document_dir().unwrap().to_str().unwrap()
        );
        assert_eq!(
            get_known_folder("FOLDERID_LocalAppData").unwrap(),
            config_local_dir().unwrap().to_str().unwrap()
        );
        assert_eq!(
            get_known_folder("FOLDERID_RoamingAppData").unwrap(),
            config_dir().unwrap().to_str().unwrap()
        );
        assert!(get_known_folder("FOLDERID_Desktop2").is_err());
    }
}
