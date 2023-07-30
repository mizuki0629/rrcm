//! Windows specific path utilities
//! This module is enabled only on windows
//! because Windows has different path format
//! (e.g. C:\Users\username\...)
//! and different path separator (e.g. C:\Users\username\foo\bar.txt)
use anyhow::Context as _;
use anyhow::{bail, Ok, Result};
use dirs;
use std::path::PathBuf;

/// Get path of known folder
/// KNOWNFOLDERID is one of the following:
/// - FOLDERID_Desktop
/// - FOLDERID_Documents
/// - FOLDERID_LocalAppData
/// - FOLDERID_RoamingAppData
/// - FOLDERID_ProgramData
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
fn get_known_folder(s: &str) -> Result<String> {
    let mut path = PathBuf::new();
    match s {
        "FOLDERID_Desktop" => {
            path.push(dirs::desktop_dir().unwrap());
        }
        "FOLDERID_Documents" => {
            path.push(dirs::document_dir().unwrap());
        }
        "FOLDERID_LocalAppData" => {
            path.push(dirs::data_local_dir().unwrap());
        }
        "FOLDERID_RoamingAppData" => {
            path.push(dirs::data_dir().unwrap());
        }
        "FOLDERID_ProgramData" => {
            path.push(dirs::data_local_dir().unwrap());
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
/// - FOLDERID_ProgramData
/// - FOLDERID_Profiles
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
fn is_known_folder_id(s: &str) -> bool {
    match s {
        "FOLDERID_Desktop" => true,
        "FOLDERID_Documents" => true,
        "FOLDERID_LocalAppData" => true,
        "FOLDERID_RoamingAppData" => true,
        "FOLDERID_ProgramData" => true,
        _ => false,
    }
}

/// Expand environment variable on windows
/// %VARNAME% format is supported
/// If VARNAME is KNOWNFOLDERID, expand to path of known folder
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
pub fn expand_env_var(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars();
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
