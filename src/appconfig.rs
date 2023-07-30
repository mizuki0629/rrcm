use crate::path::expand_env_var;
use anyhow::{bail, ensure, Context as _, Ok, Result};
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

        Self { deploy }
    }
}

pub fn init_config<P>(path: P) -> Result<AppConfig>
where
    P: AsRef<Path>,
{
    let config_path = path.as_ref().join("rrcm.toml");
    Ok(confy::load_path(&config_path)
        .with_context(|| format!("Failed to init {:?}", &config_path))?)
}

/// Load config from config.toml
pub fn load_config<P>(path: P) -> Result<AppConfig>
where
    P: AsRef<Path>,
{
    let config_path = path.as_ref().join("rrcm.toml");
    ensure!(
        config_path.exists(),
        "{:?} is not managed directory.",
        path.as_ref()
    );
    Ok(confy::load_path(&config_path)
        .with_context(|| format!("Failed to load {:?}", &config_path))?)
}
