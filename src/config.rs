use crate::path::expand_env_var;
use anyhow::ensure;
use anyhow::{bail, Ok, Result};
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct OsPath {
    pub windows: Option<String>,
    pub mac: Option<String>,
    pub linux: Option<String>,
}
impl OsPath {
    pub fn to_pathbuf(&self) -> Result<PathBuf> {
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
    pub deploy: BTreeMap<String, OsPath>,
    pub repos: BTreeMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let dotfiles = OsPath {
            windows: Some("%USERPROFILE%\\dotfiles".to_string()),
            mac: Some("${HOME}/.dotfiles".to_string()),
            linux: Some("${HOME}/.dotfiles".to_string()),
        };

        let deploy = btreemap!(
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

        let repos = BTreeMap::new();

        Self {
            dotfiles,
            deploy,
            repos,
        }
    }
}

impl AppConfig {
    pub fn to_pathbuf(&self) -> Result<PathBuf> {
        self.dotfiles.to_pathbuf()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repo {
    pub url: String,
}

pub fn load_app_config() -> Result<AppConfig> {
    Ok(confy::load("rrcm", "config")?)
}

pub fn load_app_config_with_path(path: &PathBuf) -> Result<AppConfig> {
    ensure!(path.exists(), format!("{} does not exist.", path.display()));
    ensure!(path.is_file(), format!("{} is not a file.", path.display()));
    Ok(confy::load_path(path)?)
}
