use crate::path::expand_env_var;
use anyhow::ensure;
use anyhow::{bail, Ok, Result};
use indexmap::{indexmap, IndexMap};
use reqwest;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use url::Url;

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
    pub deploy: IndexMap<String, OsPath>,
    pub repos: IndexMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let dotfiles = OsPath {
            windows: Some("%USERPROFILE%\\dotfiles".to_string()),
            mac: Some("${HOME}/.dotfiles".to_string()),
            linux: Some("${HOME}/.dotfiles".to_string()),
        };

        let deploy = indexmap!(
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

        let repos = IndexMap::new();

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

pub fn init_app_config<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    ensure!(
        !path.exists(),
        format!("{} already exists.", path.display())
    );
    let config = AppConfig::default();
    confy::store_path(path, &config)?;
    Ok(())
}

pub fn download_app_config<P>(path: P, url: Url) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    ensure!(
        !path.exists(),
        format!("{} already exists.", path.display())
    );
    let res = reqwest::blocking::get(url)?.error_for_status()?;
    std::fs::write(path, res.bytes()?)?;
    Ok(())
}

pub fn load_app_config<P>(path: P) -> Result<AppConfig>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    ensure!(path.exists(), format!("{} does not exist.", path.display()));
    Ok(confy::load_path(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config() {
        let config = AppConfig::default();
        assert_eq!(
            config.dotfiles.windows,
            Some("%USERPROFILE%\\dotfiles".to_string())
        );
        assert_eq!(config.dotfiles.mac, Some("${HOME}/.dotfiles".to_string()));
        assert_eq!(config.dotfiles.linux, Some("${HOME}/.dotfiles".to_string()));
        assert_eq!(config.deploy.len(), 3);
        assert_eq!(config.repos.len(), 0);
        assert_eq!(
            config.deploy.get("home").unwrap().windows,
            Some("%USERPROFILE%".to_string())
        );
        assert_eq!(
            config.deploy.get("home").unwrap().mac,
            Some("${HOME}".to_string())
        );
        assert_eq!(
            config.deploy.get("home").unwrap().linux,
            Some("${HOME}".to_string())
        );
        assert_eq!(
            config.deploy.get("config").unwrap().windows,
            Some("%FOLDERID_RoamingAppData%".to_string())
        );
        assert_eq!(
            config.deploy.get("config").unwrap().mac,
            Some("${XDG_CONFIG_HOME}".to_string())
        );
        assert_eq!(
            config.deploy.get("config").unwrap().linux,
            Some("${XDG_CONFIG_HOME}".to_string())
        );
        assert_eq!(
            config.deploy.get("config_local").unwrap().windows,
            Some("%FOLDERID_LocalAppData%".to_string())
        );
        assert_eq!(
            config.deploy.get("config_local").unwrap().mac,
            Some("${XDG_CONFIG_HOME}".to_string())
        );
        assert_eq!(
            config.deploy.get("config_local").unwrap().linux,
            Some("${XDG_CONFIG_HOME}".to_string())
        );
    }

    #[test]
    fn test_os_path() {
        let os_path = OsPath {
            windows: Some("%USERPROFILE%\\dotfiles".to_string()),
            mac: Some("${HOME}/.dotfiles".to_string()),
            linux: Some("${HOME}/.dotfiles".to_string()),
        };
        assert_eq!(
            os_path.to_pathbuf().unwrap(),
            if cfg!(target_os = "windows") {
                PathBuf::from(std::env::var("USERPROFILE").unwrap() + "\\dotfiles")
            } else {
                PathBuf::from(std::env::var("HOME").unwrap() + "/.dotfiles")
            }
        );
    }
}
