use anyhow::Context as _;
use anyhow::{bail, Ok, Result};
use dirs;
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// # get_xdg_default
///
/// Get default path of XDG Base Directory
/// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
/// | Variable | Default Value |
/// |----------|---------------|
/// | XDG_CONFIG_HOME | $HOME/.config |
/// | XDG_DATA_HOME | $HOME/.local/share |
/// | XDG_CACHE_HOME | $HOME/.cache |
/// | XDG_RUNTIME_DIR | /run/user/$UID |
/// | XDG_STATE_HOME | $HOME/.local/state |
///
/// ## Example
/// ```
/// use appconfig::get_xdg_default;
/// assert_eq!(get_xdg_default("XDG_CONFIG_HOME"), "$HOME/.config");
/// assert_eq!(get_xdg_default("XDG_DATA_HOME"), "$HOME/.local/share");
/// assert_eq!(get_xdg_default("XDG_CACHE_HOME"), "$HOME/.cache");
/// assert_eq!(get_xdg_default("XDG_RUNTIME_DIR"), "/run/user/$UID");
/// assert_eq!(get_xdg_default("XDG_STATE_HOME"), "$HOME/.local/state");
/// ```
///
/// ## Panics
/// When invalid XDG Base Directory is specified
/// ```
/// use appconfig::get_xdg_default;
/// get_xdg_default("XDG_CONFIG_DIR"); // panic
/// ```
///
fn get_xdg_default(s: &str) -> Result<String> {
    let mut path = PathBuf::new();
    match s {
        "XDG_CONFIG_HOME" => {
            path.push(dirs::config_dir().unwrap());
        }
        "XDG_DATA_HOME" => {
            path.push(dirs::data_dir().unwrap());
        }
        "XDG_CACHE_HOME" => {
            path.push(dirs::cache_dir().unwrap());
        }
        "XDG_RUNTIME_DIR" => {
            path.push(dirs::runtime_dir().unwrap());
        }
        "XDG_STATE_HOME" => {
            path.push(dirs::state_dir().unwrap());
        }
        _ => bail!("invalid XDG Base Directory: {}", s),
    }
    Ok(path.to_str().unwrap().to_string())
}

/// # is_XDG_Base_Directory
///
/// Check if the specified string is XDG Base Directory
/// XDG Base Directory is one of the following:
/// - XDG_CONFIG_HOME
/// - XDG_DATA_HOME
/// - XDG_CACHE_HOME
/// - XDG_RUNTIME_DIR
/// - XDG_STATE_HOME
///
/// ## Example
/// ```
/// use appconfig::is_XDG_Base_Directory;
/// assert_eq!(is_xdg_base_directory("XDG_CONFIG_HOME"), true);
/// assert_eq!(is_xdg_base_directory("XDG_DATA_HOME"), true);
/// assert_eq!(is_xdg_base_directory("XDG_CACHE_HOME"), true);
/// assert_eq!(is_xdg_base_directory("XDG_RUNTIME_DIR"), true);
/// assert_eq!(is_xdg_base_directory("XDG_STATE_HOME"), true);
/// assert_eq!(is_xdg_base_directory("XDG_CONFIG_DIR"), false);
/// ```
///
fn is_xdg_base_directory(s: &str) -> bool {
    match s {
        "XDG_CONFIG_HOME" => true,
        "XDG_DATA_HOME" => true,
        "XDG_CACHE_HOME" => true,
        "XDG_RUNTIME_DIR" => true,
        "XDG_STATE_HOME" => true,
        _ => false,
    }
}

/// # get_known_folder
///
/// Get path of known folder
/// KNOWNFOLDERID is one of the following:
/// - FOLDERID_Desktop
/// - FOLDERID_Documents
/// - FOLDERID_LocalAppData
/// - FOLDERID_RoamingAppData
/// - FOLDERID_ProgramData
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
///
#[cfg(target_os = "windows")]
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

/// # is_known_folder_id
///
/// Check if the specified string is knownfolderid
/// KNOWNFOLDERID is one of the following:
/// - FOLDERID_Desktop
/// - FOLDERID_Documents
/// - FOLDERID_LocalAppData
/// - FOLDERID_RoamingAppData
/// - FOLDERID_ProgramData
/// - FOLDERID_Profiles
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
///
/// ## Example
/// ```
/// use appconfig::is_known_folder_id;
/// assert_eq!(is_known_folder_id("FOLDERID_Desktop"), true);
/// assert_eq!(is_known_folder_id("FOLDERID_Documents"), true);
/// assert_eq!(is_known_folder_id("FOLDERID_LocalAppData"), true);
/// assert_eq!(is_known_folder_id("FOLDERID_RoamingAppData"), true);
/// assert_eq!(is_known_folder_id("FOLDERID_ProgramData"), true);
/// assert_eq!(is_known_folder_id("FOLDERID_Desktop2"), false);
/// ```
///
#[cfg(target_os = "windows")]
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

/// # expand_env_var_windows
///
/// Expand environment variable on windows
/// %VARNAME% format is supported
/// If VARNAME is KNOWNFOLDERID, expand to path of known folder
/// https://docs.microsoft.com/ja-jp/windows/win32/shell/knownfolderid
///
#[cfg(target_os = "windows")]
fn expand_env_var_windows(s: &str) -> Result<String> {
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
            if !is_known_folder_id(&varname) {
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

/// # expand_env_var
///
/// Expand environment variable in String
/// ${VARNAME} format is expanded
/// If VARNAME is XDG Base Directory, it is expanded to the path of the XDG Base Directory
/// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
///
fn expand_env_var_unix(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(c) = chars.next() {
                if c == '{' {
                    let mut varname = String::new();
                    let mut is_end = false;
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            is_end = true;
                            break;
                        }
                        varname.push(c);
                    }
                    if !is_end {
                        bail!("invalid env var name: {}", varname);
                    }
                    if varname.is_empty() {
                        bail!("invalid env var name: {}", varname);
                    }
                    if is_xdg_base_directory(&varname) {
                        result.push_str(&get_xdg_default(&varname)?);
                        continue;
                    }
                    let value = std::env::var(&varname)
                        .with_context(|| format!("env var {} not found", varname))?;
                    result.push_str(&value);
                } else {
                    result.push('$');
                    result.push(c);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

/// # expand_env_var
///
/// Expand environment variable in string
fn expand_env_var(s: &str) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        expand_env_var_windows(s)
    }

    #[cfg(not(target_os = "windows"))]
    {
        expand_env_var_unix(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_var() {
        assert_eq!(
            expand_env_var("${HOME}").unwrap(),
            std::env::var("HOME").unwrap()
        );
    }

    #[test]
    fn test_expand_env_var2() {
        assert_eq!(
            expand_env_var("${HOME}/bin").unwrap(),
            format!("{}/bin", std::env::var("HOME").unwrap())
        );
    }

    #[test]
    fn test_expand_env_var3() {
        assert_eq!(
            expand_env_var("/abc/${HOME}/bin").unwrap(),
            format!("/abc/{}/bin", std::env::var("HOME").unwrap())
        );
    }

    #[test]
    fn test_expand_env_var4() {
        assert_eq!(
            expand_env_var("/abc/${HOME}/bin/${HOME}").unwrap(),
            format!(
                "/abc/{}/bin/{}",
                std::env::var("HOME").unwrap(),
                std::env::var("HOME").unwrap()
            )
        );
    }

    #[test]
    fn test_expand_env_var5() {
        assert!(expand_env_var("${UNKNOWN}").is_err());
    }

    #[test]
    fn test_expand_env_var6() {
        assert!(expand_env_var("${}").is_err());
    }

    #[test]
    fn test_expand_env_var7() {
        assert!(expand_env_var("${HOME").is_err());
    }

    #[test]
    fn test_expand_env_var9() {
        assert_eq!(
            expand_env_var("${XDG_CONFIG_HOME}").unwrap(),
            dirs::config_dir().unwrap().to_str().unwrap().to_string()
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsPath {
    windows: Option<String>,
    mac: Option<String>,
    linux: Option<String>,
}
impl OsPath {
    pub fn to_pathbuf(self: &Self) -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let Some(path) = &self.windows else {
                bail!("Windows Path not defined.");
            };
            Ok(PathBuf::from(expand_env_var(path)?))
        }

        #[cfg(target_os = "macos")]
        {
            let Some(path) = &self.mac else {
                bail!("Mac Path not defined.");
            };
            Ok(PathBuf::from(expand_env_var(path)?))
        }

        #[cfg(target_os = "linux")]
        {
            let Some(path) = &self.linux else {
                bail!("Linux Path not defined.");
            };
            Ok(PathBuf::from(expand_env_var(path)?))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub dotfiles: OsPath,
    pub deploy: HashMap<String, OsPath>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let deploy = hashmap!(
            String::from("home") => OsPath {
                windows: Some("%USERPROFILE%".to_string()),
                mac: Some("${HOME}".to_string()),
                linux: Some("${HOME}".to_string()),
            },
            String::from("config") => OsPath {
                windows: Some("%FOLDERID_RoamingAppData%".to_string()),
                mac: Some("${XDG_CONFIG_HOME}".to_string()),
                linux: Some("${XDG_CONFIG_HOME}".to_string()),
            },
            String::from("config_local") => OsPath {
                windows: Some("%FOLDERID_LocalAppData%".to_string()),
                mac: Some("${XDG_CONFIG_HOME}".to_string()),
                linux: Some("${XDG_CONFIG_HOME}".to_string()),
            },
        );

        let dotfiles = OsPath {
            windows: Some("%USERPROFILE%\\dotfiles".to_string()),
            mac: Some("${HOME}/dotfiles".to_string()),
            linux: Some("${HOME}/dotfiles".to_string()),
        };

        Self { dotfiles, deploy }
    }
}

/// # load_config
/// Load config from config.toml
pub fn load_config() -> Result<AppConfig> {
    let pkg_name = env!("CARGO_PKG_NAME");
    Ok(confy::load_path(
        dirs::config_local_dir()
            .unwrap()
            .join(pkg_name)
            .join("config.toml"),
    )?)
}
