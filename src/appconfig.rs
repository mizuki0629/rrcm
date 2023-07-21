use anyhow::Context as _;
use anyhow::{bail, ensure, Ok, Result};
use dirs;
use maplit::hashmap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub trait ToPathBuf {
    fn to_pathbuf(self: &Self) -> Result<PathBuf>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MacOSStandardDirectories {
    HOME,
    ApplicationSupport,
    Preference,
}
impl ToPathBuf for MacOSStandardDirectories {
    fn to_pathbuf(self: &Self) -> Result<PathBuf> {
        Ok(match self {
            MacOSStandardDirectories::HOME => dirs::home_dir(),
            MacOSStandardDirectories::ApplicationSupport => dirs::config_dir(),
            MacOSStandardDirectories::Preference => dirs::preference_dir(),
        }
        .with_context(|| format!("{:?} cant use.", self))?)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum XDGBaseDirectory {
    HOME,
    XDG_CACHE_HOME,
    XDG_CONFIG_HOME,
    XDG_DATA_HOME,
    XDG_BIN_HOME,
    XDG_RUNTIME_DIR,
    XDG_STATE_HOME,
    XDG_MUSIC_DIR,
    XDG_DESKTOP_DIR,
    XDG_DOCUMENTS_DIR,
    XDG_DOWNLOAD_DIR,
    XDG_PICTURE_DIR,
    XDG_PUBLICSHARE_DIR,
    XDG_TEMPLATES_DIR,
    XDG_VIDEOS_DIR,
}
impl ToPathBuf for XDGBaseDirectory {
    fn to_pathbuf(self: &Self) -> Result<PathBuf> {
        Ok(match self {
            XDGBaseDirectory::HOME => dirs::home_dir(),
            XDGBaseDirectory::XDG_CACHE_HOME => dirs::cache_dir(),
            XDGBaseDirectory::XDG_CONFIG_HOME => dirs::config_dir(),
            XDGBaseDirectory::XDG_DATA_HOME => dirs::data_dir(),
            XDGBaseDirectory::XDG_BIN_HOME => dirs::executable_dir(),
            XDGBaseDirectory::XDG_RUNTIME_DIR => dirs::runtime_dir(),
            XDGBaseDirectory::XDG_STATE_HOME => dirs::state_dir(),
            XDGBaseDirectory::XDG_MUSIC_DIR => dirs::audio_dir(),
            XDGBaseDirectory::XDG_DESKTOP_DIR => dirs::desktop_dir(),
            XDGBaseDirectory::XDG_DOCUMENTS_DIR => dirs::document_dir(),
            XDGBaseDirectory::XDG_DOWNLOAD_DIR => dirs::download_dir(),
            XDGBaseDirectory::XDG_PICTURE_DIR => dirs::picture_dir(),
            XDGBaseDirectory::XDG_PUBLICSHARE_DIR => dirs::public_dir(),
            XDGBaseDirectory::XDG_TEMPLATES_DIR => dirs::template_dir(),
            XDGBaseDirectory::XDG_VIDEOS_DIR => dirs::video_dir(),
        }
        .with_context(|| format!("{:?} cant use.", self))?)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum KnownFolder {
    FOLDERID_Profile,
    FOLDERID_RoamingAppData,
    FOLDERID_LocalAppData,
    FOLDERID_Documents,
    FOLDERID_Templates,
}
impl ToPathBuf for KnownFolder {
    fn to_pathbuf(self: &Self) -> Result<PathBuf> {
        Ok(match self {
            KnownFolder::FOLDERID_Profile => dirs::home_dir(),
            KnownFolder::FOLDERID_RoamingAppData => dirs::config_dir(),
            KnownFolder::FOLDERID_LocalAppData => dirs::config_local_dir(),
            KnownFolder::FOLDERID_Documents => dirs::document_dir(),
            KnownFolder::FOLDERID_Templates => dirs::template_dir(),
        }
        .with_context(|| format!("{:?} cant use.", self))?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OsPathType<T>
where
    T: ToPathBuf,
{
    Absolute { path: PathBuf },
    Relative { base: T, path: Option<PathBuf> },
}
impl<T> OsPathType<T>
where
    T: ToPathBuf,
{
    fn to_pathbuf(self: &Self) -> Result<PathBuf> {
        let pathbuf = match self {
            OsPathType::Absolute { path } => path.clone(),
            OsPathType::Relative { base, path } => {
                let mut base = base.to_pathbuf()?;
                if let Some(path) = path {
                    base.push(path)
                }
                base
            }
        };

        ensure!(pathbuf.exists(), "dir not exists: {:?} ", pathbuf);

        Ok(pathbuf.clone())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OsPath {
    windows: Option<OsPathType<KnownFolder>>,
    mac: Option<OsPathType<MacOSStandardDirectories>>,
    linux: Option<OsPathType<XDGBaseDirectory>>,
}
impl OsPath {
    pub fn to_pathbuf(self: &Self) -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        let Some(path) = &self.windows else {
            bail!("Windows Path not defined.");
        };

        #[cfg(target_os = "linux")]
        let Some(path) = &self.linux else {
            bail!("Linux Path not defined.");
        };

        Ok(path.to_pathbuf()?)
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
                windows: Some(OsPathType::Relative {
                    base: KnownFolder::FOLDERID_Profile,
                    path: None,
                }),
                mac: Some(OsPathType::Relative {
                    base: MacOSStandardDirectories::HOME,
                    path: None,
                }),
                linux: Some(OsPathType::Relative {
                    base: XDGBaseDirectory::HOME,
                    path: None,
                }),
            },
            String::from("config") => OsPath {
                windows: Some(OsPathType::Relative {
                    base: KnownFolder::FOLDERID_RoamingAppData,
                    path: None,
                }),
                mac: Some(OsPathType::Relative {
                    base: MacOSStandardDirectories::Preference,
                    path: None,
                }),
                linux: Some(OsPathType::Relative {
                    base: XDGBaseDirectory::XDG_CONFIG_HOME,
                    path: None,
                }),
            },
            String::from("config_local") => OsPath {
                windows: Some(OsPathType::Relative {
                    base: KnownFolder::FOLDERID_LocalAppData,
                    path: None,
                }),
                mac: Some(OsPathType::Relative {
                    base: MacOSStandardDirectories::Preference,
                    path: None,
                }),
                linux: Some(OsPathType::Relative {
                    base: XDGBaseDirectory::XDG_CONFIG_HOME,
                    path: None,
                }),
            },
        );

        let dotfiles = OsPath {
            windows: Some(OsPathType::Relative {
                base: KnownFolder::FOLDERID_Profile,
                path: Some(PathBuf::from("dotfiles")),
            }),
            mac: Some(OsPathType::Relative {
                base: MacOSStandardDirectories::HOME,
                path: Some(PathBuf::from("dotfiles")),
            }),
            linux: Some(OsPathType::Relative {
                base: XDGBaseDirectory::HOME,
                path: Some(PathBuf::from("dotfiles")),
            }),
        };

        Self { dotfiles, deploy }
    }
}

pub fn load_config() -> Result<AppConfig> {
    let pkg_name = env!("CARGO_PKG_NAME");
    Ok(confy::load_path(
        dirs::config_local_dir()
            .unwrap()
            .join(pkg_name)
            .join("config.yaml"),
    )?)
}

