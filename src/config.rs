use crate::path::expand_env_var;
use anyhow::ensure;
use anyhow::{bail, Ok, Result};
use indexmap::IndexMap;
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
pub struct Repository {
    pub name: String,
    pub url: String,
    pub deploy: IndexMap<String, OsPath>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub dotfiles: OsPath,
    pub repos: Vec<Repository>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let dotfiles = OsPath {
            windows: Some("%USERPROFILE%\\dotfiles".to_string()),
            mac: Some("${HOME}/.dotfiles".to_string()),
            linux: Some("${HOME}/.dotfiles".to_string()),
        };

        let repos = Vec::new();
        Self { dotfiles, repos }
    }
}

impl AppConfig {
    pub fn to_pathbuf(&self) -> Result<PathBuf> {
        self.dotfiles.to_pathbuf()
    }
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
        assert_eq!(config.repos.len(), 0);
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
