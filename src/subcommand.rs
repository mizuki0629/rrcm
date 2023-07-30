//! Subcommand module
//!
//! This module contains subcommands.
//! Each subcommand is implemented as a function.
use crate::appconfig;
use crate::fs;
use anyhow::{bail, Context as _, Ok, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use std::fs::{read_dir, read_link};
use std::path::{Path, PathBuf};

#[cfg(target_family = "unix")]
use termion::color;

fn create_filemap(path: &Path) -> Result<HashMap<OsString, PathBuf>> {
    let mut dst: HashMap<_, _> = HashMap::new();
    for entry in read_dir(path)?.filter_map(|e| e.ok()) {
        dst.insert(entry.file_name(), entry.path());
    }
    Ok(dst)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum DeployStatus {
    UnDeployed,
    Deployed,
    Conflict,
    UnManaged,
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

    if to.as_ref().is_symlink() && from.as_ref() == read_link(to).unwrap() {
        return DeployStatus::Deployed;
    } else {
        return DeployStatus::Conflict;
    }
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
                DeployStatus::Conflict => {
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
        DeployStatus::Conflict => {
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
    println!("Deploy From -> To ");
    for deploy_path in deploy_paths {
        println!("    {:?} -> {:?}", deploy_path.from, deploy_path.to);
    }
    println!("");
}

fn print_status(lookup: &HashMap<DeployStatus, Vec<String>>, status: DeployStatus) -> Result<()> {
    if let Some(l) = lookup.get(&status) {
        for ff in l {
            #[cfg(target_family = "unix")]
            println!(
                "{}{:>12} {}{:}",
                match status {
                    DeployStatus::Deployed => color::Fg(color::Green).to_string(),
                    DeployStatus::UnDeployed => color::Fg(color::Yellow).to_string(),
                    DeployStatus::UnManaged =>
                        color::Fg(color::AnsiValue::grayscale(12)).to_string(),
                    DeployStatus::Conflict => color::Fg(color::Red).to_string(),
                },
                format!("{:?}", status),
                color::Fg(color::Reset),
                ff
            );

            #[cfg(not(target_family = "unix"))]
            println!("{:>12} {:}", format!("{:?}", status), ff);
        }
    }

    return Ok(());
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
        .map(|deploy_path| {
            let from_files = create_filemap(&deploy_path.from).unwrap();
            let to_files = create_filemap(&deploy_path.to).unwrap();

            from_files
                .keys()
                .chain(to_files.keys())
                .collect::<BTreeSet<_>>()
                .iter()
                .map(|f| {
                    let from = from_files.get(*f);
                    let to = to_files.get(*f);
                    let s = get_status_impl(from, to);
                    let ff = match s {
                        DeployStatus::Deployed => format!(
                            "{:} -> {:}",
                            from.unwrap().to_str().unwrap(),
                            to.unwrap().to_str().unwrap()
                        ),
                        DeployStatus::UnDeployed => format!("{:}", from.unwrap().to_str().unwrap()),
                        DeployStatus::UnManaged => format!("{:}", to.unwrap().to_str().unwrap()),
                        DeployStatus::Conflict => format!("{:}", from.unwrap().to_str().unwrap()),
                    };

                    (s, ff)
                })
                .collect_vec()
        })
        .flatten()
        .into_group_map();

    for s in vec![
        DeployStatus::Deployed,
        DeployStatus::UnDeployed,
        DeployStatus::Conflict,
    ] {
        if lookup.contains_key(&s) {
            print_status_description(&s);
            print_status(&lookup, s)?;
            println!("");
        }
    }
    return Ok(());
}

pub fn init<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    appconfig::init_config(&path)?;
    return Ok(());
}
