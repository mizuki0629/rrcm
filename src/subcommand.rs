use crate::appconfig;
use crate::fs;
use anyhow::Context as _;
use anyhow::{bail, Ok, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use std::fs::{read_dir, read_link, rename};
use std::path;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_family = "unix")]
use termion::color;

fn create_filemap(path: &path::Path) -> Result<HashMap<OsString, PathBuf>> {
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

fn get_deploy_paths(app_config: &appconfig::AppConfig) -> Result<Vec<DeployPath>> {
    let dotfiles_path = app_config
        .dotfiles
        .to_pathbuf()
        .context("dotfiles directory path not defined.")?;

    Ok(app_config
        .deploy
        .iter()
        .filter_map(|(dirname, to)| {
            let from = dotfiles_path.join(PathBuf::from(dirname));
            if !from.exists() {
                return None;
            }

            let to = to.to_pathbuf().ok()?;
            Some(DeployPath { from, to })
        })
        .collect::<Vec<DeployPath>>())
}

pub fn deploy<P>(path: P, force: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let from = path.as_ref();
    let abs_from = fs::absolutize(from)?;

    let app_config = appconfig::load_config()?;
    for deploy_path in get_deploy_paths(&app_config)? {
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
                    DeployStatus::Deployed => format!("{}", color::Fg(color::Green)),
                    DeployStatus::UnDeployed => format!("{}", color::Fg(color::Yellow)),
                    DeployStatus::UnManaged =>
                        format!("{}", color::Fg(color::AnsiValue::grayscale(12))),
                    DeployStatus::Conflict => format!("{}", color::Fg(color::Red)),
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

pub fn status(all: bool) -> Result<()> {
    let app_config = appconfig::load_config()?;
    let deploy_paths = get_deploy_paths(&app_config)?;
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
