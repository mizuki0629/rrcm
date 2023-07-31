//! Subcommand module
//!
//! This module contains subcommands.
//! Each subcommand is implemented as a function.
use crate::appconfig;
use crate::fs;
use anyhow::{bail, Context as _, Ok, Result};
use core::fmt::{self, Display};
use core::hash::Hash;
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_dir, read_link};
use std::io::stdout;
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, Clone)]
enum DeployStatus {
    UnDeployed,
    Deployed,
    Conflict { cause: String },
    UnManaged,
}
impl PartialEq for DeployStatus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (DeployStatus::UnDeployed, DeployStatus::UnDeployed)
                | (DeployStatus::Deployed, DeployStatus::Deployed)
                | (DeployStatus::UnManaged, DeployStatus::UnManaged)
                | (DeployStatus::Conflict { .. }, DeployStatus::Conflict { .. })
        )
    }
}

impl Hash for DeployStatus {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DeployStatus::UnDeployed => 0.hash(state),
            DeployStatus::Deployed => 1.hash(state),
            DeployStatus::UnManaged => 2.hash(state),
            DeployStatus::Conflict { .. } => 3.hash(state),
        }
    }
}

impl Display for DeployStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeployStatus::UnDeployed => write!(f, "UnDeployed"),
            DeployStatus::Deployed => write!(f, "Deployed"),
            DeployStatus::UnManaged => write!(f, "UnManaged"),
            DeployStatus::Conflict { .. } => write!(f, "Conflict"),
        }
    }
}

fn get_status<P, Q>(from: P, to: Q) -> DeployStatus
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    get_status_impl(
        from.as_ref().exists().then_some(from),
        to.as_ref().exists().then_some(to),
    )
}

// 存在するか       Y Y Y N
// Link             Y Y N /
// Link先が正しいか Y N / /
// ------------------------
//                  T F F F
fn get_status_impl<P, Q>(from: Option<P>, to: Option<Q>) -> DeployStatus
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let Some(from) = from else {
        return DeployStatus::UnManaged;
    };
    let Some(to) = to else {
        return DeployStatus::UnDeployed;
    };

    if !to.as_ref().is_symlink() {
        return DeployStatus::Conflict {
            cause: "Not symlink.".to_string(),
        };
    }

    let abs_to_link = fs::absolutize(read_link(to).unwrap()).unwrap();
    if fs::absolutize(from).unwrap() != abs_to_link {
        return DeployStatus::Conflict {
            cause: format!("Different symlink to {}.", abs_to_link.to_string_lossy()),
        };
    }

    DeployStatus::Deployed
}

#[derive(Debug, Serialize, Deserialize)]
struct DeployPath {
    from: PathBuf,
    to: PathBuf,
}

fn get_deploy_paths<P>(path: P, app_config: &appconfig::AppConfig) -> Result<Vec<DeployPath>>
where
    P: AsRef<Path>,
{
    Ok(app_config
        .deploy
        .iter()
        .filter_map(|(dirname, to)| {
            let from = path.as_ref().join(PathBuf::from(dirname));
            if !from.exists() {
                return None;
            }

            let to = to.to_pathbuf().ok()?;
            Some(DeployPath { from, to })
        })
        .collect::<Vec<DeployPath>>())
}

/// Deploy Files
///
/// # Arguments
/// * `path` - Path to deploy files
/// * `force` - Force deploy
///
/// # Example
/// ```sh
/// $ rrcm deploy .vimrc
/// ```
///
pub fn deploy<P>(path: P, force: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let from = path.as_ref();
    let abs_from = fs::absolutize(from)?;

    let managed_path = path
        .as_ref()
        .parent()
        .with_context(|| format!("Can not get parent directory of {:?}", path.as_ref()))?;
    let app_config = appconfig::load_config(managed_path)?;
    for deploy_path in get_deploy_paths(managed_path, &app_config)? {
        if deploy_path.from == abs_from.parent().unwrap() {
            let to = deploy_path.to.join(from.file_name().unwrap());

            match get_status(from, &to) {
                DeployStatus::UnDeployed => {
                    fs::symlink(from, &to)?;
                    return Ok(());
                }
                DeployStatus::Deployed => {
                    return Ok(());
                }
                DeployStatus::Conflict { .. } => {
                    if force {
                        fs::remove(&to)?;
                        fs::symlink(from, &to)?;
                        return Ok(());
                    }
                    bail!("Another file exists. {:?}", to);
                }
                DeployStatus::UnManaged => {
                    bail!("File not exists. {:?}", to);
                }
            }

            #[allow(unreachable_code)]
            {
                unreachable!("The loop should always return");
            }
        }
    }
    bail!("{:?} is not source directory.", abs_from);
}

fn print_status_description(s: &DeployStatus) {
    let pkg_name = env!("CARGO_PKG_NAME");
    match s {
        DeployStatus::Deployed => {
            println!("Files deployed:");
        }
        DeployStatus::UnDeployed => {
            println!("Files can deploy:");
            println!("  (use \"{:} deploy <PATH>...\" to deploy files)", pkg_name);
        }
        DeployStatus::UnManaged => {
            println!("Files are not managed:");
            println!(
                "  (use \"{:} add <PATH>...\" to manage and deploy files)",
                pkg_name
            );
        }
        DeployStatus::Conflict { .. } => {
            println!("Files can not deploy:");
            println!("  (already exists other file at deploy path.)");
            println!("  (you shuld move or delete file manualiy or )");
            println!(
                "  (use \"{:} deploy -f <PATH>...\" force deploy files)",
                pkg_name
            );
        }
    }
}

fn print_deploy_paths(deploy_paths: &Vec<DeployPath>) {
    println!("Deploy From => To ");
    for deploy_path in deploy_paths {
        println!(
            "    {} => {}",
            deploy_path.from.to_string_lossy(),
            deploy_path.to.to_string_lossy()
        );
    }
    println!();
}

fn print_status(lookup: &HashMap<DeployStatus, Vec<String>>, status: DeployStatus) -> Result<()> {
    if let Some(l) = lookup.get(&status) {
        for ff in l {
            stdout()
                .execute(match status {
                    DeployStatus::Deployed => SetForegroundColor(Color::Green),
                    DeployStatus::UnDeployed => SetForegroundColor(Color::Yellow),
                    DeployStatus::Conflict { .. } => SetForegroundColor(Color::Red),
                    DeployStatus::UnManaged => SetForegroundColor(Color::Grey),
                })?
                .execute(Print(format!("{:>12}", format!("{:}", status))))?
                .execute(ResetColor)?
                .execute(Print(format!(" {}\n", ff)))?;
        }
    }

    Ok(())
}

/// Show status of files.
/// # Arguments
/// * `all` - Show all files.
/// # Example
/// ```sh
/// $ rrcm status
/// ```
/// ```sh
/// $ rrcm status -a
/// ```
pub fn status<P>(path: P, _all: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let app_config = appconfig::load_config(&path)?;
    let deploy_paths = get_deploy_paths(path, &app_config)?;
    print_deploy_paths(&deploy_paths);

    let lookup = deploy_paths
        .iter()
        .filter_map(|deploy_path| {
            Some(
                read_dir(&deploy_path.from)
                    .ok()?
                    .filter_map(|from| {
                        let from = from.ok()?.path();
                        let to = deploy_path.to.join(from.file_name()?);
                        let s = get_status(&from, &to);
                        let ff = match &s {
                            DeployStatus::Deployed => {
                                format!("{:} => {:}", from.to_string_lossy(), to.to_string_lossy())
                            }
                            DeployStatus::UnDeployed => format!("{:}", from.to_string_lossy()),
                            DeployStatus::UnManaged => format!("{:}", to.to_string_lossy()),
                            DeployStatus::Conflict { cause } => format!(
                                "{:} => {:} ({:})",
                                from.to_string_lossy(),
                                to.to_string_lossy(),
                                cause,
                            ),
                        };

                        Some((s, ff))
                    })
                    .collect_vec(),
            )
        })
        .flatten()
        .into_group_map();

    for s in vec![
        DeployStatus::Deployed,
        DeployStatus::UnDeployed,
        DeployStatus::Conflict {
            cause: "".to_string(),
        },
    ] {
        if lookup.contains_key(&s) {
            print_status_description(&s);
            print_status(&lookup, s)?;
            println!();
        }
    }
    Ok(())
}

pub fn init<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    appconfig::init_config(&path)?;
    Ok(())
}
