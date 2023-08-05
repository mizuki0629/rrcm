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
    use rstest::rstest;

    #[rstest]
    #[case("FOLDERID_Desktop", true)]
    #[case("FOLDERID_Documents", true)]
    #[case("FOLDERID_LocalAppData", true)]
    #[case("FOLDERID_RoamingAppData", true)]
    #[case("FOLDERID_Desktop2", false)]
    fn test_is_known_folder_id(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_known_folder_id(input), expected);
    }

    #[rstest]
    #[case("FOLDERID_Desktop", desktop_dir().unwrap().to_str().unwrap().to_string())]
    #[case("FOLDERID_Documents", document_dir().unwrap().to_str().unwrap().to_string())]
    #[case(
        "FOLDERID_LocalAppData",
        config_local_dir().unwrap().to_str().unwrap().to_string()
    )]
    #[case(
        "FOLDERID_RoamingAppData",
        config_dir().unwrap().to_str().unwrap().to_string()
    )]
    fn test_get_known_folder(#[case] input: &str, #[case] expected: String) {
        assert_eq!(get_known_folder(input).unwrap(), expected);
    }

    #[rstest]
    #[case("FOLDERID_Desktop2")]
    fn test_get_known_folder_error(#[case] input: &str) {
        assert!(get_known_folder(input).is_err());
    }

    #[rstest]
    #[case("%FOLDERID_Desktop%", desktop_dir().unwrap().to_str().unwrap().to_string())]
    #[case(
        "%FOLDERID_Documents%",
        document_dir().unwrap().to_str().unwrap().to_string()
    )]
    #[case(
        "%FOLDERID_LocalAppData%",
        config_local_dir().unwrap().to_str().unwrap().to_string()
    )]
    #[case(
        "%FOLDERID_RoamingAppData%",
        config_dir().unwrap().to_str().unwrap().to_string()
    )]
    #[case("%USERPROFILE%", dirs::home_dir().unwrap().to_str().unwrap().to_string())]
    #[case("%USERPROFILE%\\foo", format!("{}\\foo", dirs::home_dir().unwrap().to_str().unwrap()))]
    #[case("%USERPROFILE%\\foo\\%USERNAME%", format!("{}\\foo\\{}", dirs::home_dir().unwrap().to_str().unwrap(), std::env::var("USERNAME").unwrap()))]
    #[case("%USERPROFILE%\\foo\\%USERNAME%\\bar", format!("{}\\foo\\{}\\bar", dirs::home_dir().unwrap().to_str().unwrap(), std::env::var("USERNAME").unwrap()))]
    fn test_expand_env_var(#[case] input: &str, #[case] expected: String) {
        assert_eq!(expand_env_var(input).unwrap(), expected);
    }

    #[rstest]
    #[case("%USERPROFILE")]
    #[case("%USERPROFILE2%")]
    #[case("%%")]
    fn test_expand_env_var_error(#[case] input: &str) {
        assert!(expand_env_var(input).is_err());
    }
}
