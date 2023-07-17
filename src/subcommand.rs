use crate::fs;
use anyhow;
use dirs;
use itertools::Itertools;
use std::collections::{BTreeSet, HashMap};
use std::ffi::OsString;
use std::fs::{read_dir, read_link, rename};
use std::path;
use std::path::Path;
use std::path::PathBuf;

#[cfg(target_family = "unix")]
use termion::color;

// 存在するか       Y Y Y N
// Link             Y Y N /
// Link先が正しいか Y N / /
// ------------------------
//                  T F F F

fn create_filemap(path: &path::Path) -> anyhow::Result<HashMap<OsString, PathBuf>> {
    let mut dst: HashMap<_, _> = HashMap::new();
    for entry in read_dir(path)?.filter_map(|e| e.ok()) {
        dst.insert(entry.file_name(), entry.path());
    }
    anyhow::Ok(dst)
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum DeployStatus {
    UnDeployed,
    Deployed,
    Conflict,
    UnManaged,
}

fn get_status<P, Q>(from: Option<P>, to: Option<Q>) -> DeployStatus
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

struct DeployPath {
    from: PathBuf,
    to: PathBuf,
}

fn get_deploy_paths() -> Vec<DeployPath> {
    let mut paths = Vec::new();

    let mut from = dirs::home_dir().unwrap();
    from.push("dotfiles");
    from.push("config");
    paths.push(DeployPath {
        from,
        to: dirs::config_local_dir().unwrap(),
    });

    let mut from = dirs::home_dir().unwrap();
    from.push("dotfiles");
    from.push("home");
    paths.push(DeployPath {
        from,
        to: dirs::home_dir().unwrap(),
    });

    paths
}

pub fn deploy<P>(path: P, force: bool) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let canonicalized_path = fs::canonicalize(path)?;
    for deploy_path in get_deploy_paths() {
        if deploy_path.from == canonicalized_path.parent().unwrap() {
            let deploy_path_to = deploy_path.to.join(path.file_name().unwrap());
            if deploy_path_to.exists() {
                if force {
                    fs::remove(deploy_path_to.clone())?;
                } else {
                    return Err(anyhow::anyhow!(
                        "File exists {:}",
                        deploy_path_to.to_str().unwrap()
                    ));
                }
            }
            fs::symlink(path, deploy_path_to)?;
            return Ok(());
        }
    }
    return Err(anyhow::anyhow!(
        "{:?} is not managed directory. Please add config.",
        canonicalized_path
    ));
}

pub fn add<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let canonicalized_path = fs::canonicalize(path)?;
    for deploy_path in get_deploy_paths() {
        if deploy_path.to == canonicalized_path.parent().unwrap() {
            let manage_path = deploy_path.from.join(path.file_name().unwrap());
            if manage_path.exists() {
                return Err(anyhow::anyhow!("{:?} is already exists.", manage_path));
            }
            rename(path, manage_path.clone())?;
            fs::symlink(manage_path, path)?;
            return Ok(());
        }
    }
    return Err(anyhow::anyhow!(
        "{:?} is not managed directory. Please add config.",
        canonicalized_path
    ));
}

pub fn list(paths: Vec<PathBuf>) -> anyhow::Result<()> {
    for path in paths {
        let abs_path = fs::absolutize(&path)?;
        for deploy_path in get_deploy_paths() {
            if deploy_path.from == abs_path.parent().unwrap()
                || deploy_path.to == abs_path.parent().unwrap()
            {
                let to = deploy_path.to.join(abs_path.file_name().unwrap());
                let from = deploy_path.from.join(abs_path.file_name().unwrap());

                let s = get_status(
                    if from.exists() {
                        Some(from.as_path())
                    } else {
                        None
                    },
                    if to.exists() {
                        Some(to.as_path())
                    } else {
                        None
                    },
                );
                println!("{:?} {:?}", s, path);
            }
        }
    }
    return Ok(());
}

fn print_status_description(s: &DeployStatus) {
    let pkg_name = env!("CARGO_PKG_NAME");
    match s {
        DeployStatus::Deployed => {
            println!("Files deployed:");
        }
        DeployStatus::UnDeployed => {
            println!(
                "Files can deploy:
  (use \"{:} deploy <PATH>...\" to deploy files)",
                pkg_name
            );
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

fn print_status(
    lookup: &HashMap<DeployStatus, Vec<PathBuf>>,
    status: DeployStatus,
) -> anyhow::Result<()> {
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
                ff.to_str().unwrap()
            );

            #[cfg(not(target_family = "unix"))]
            println!(
                "{:>12} {:}",
                format!("{:?}", status),
                ff.to_str().unwrap()
            );
        }
    }

    return Ok(());
}

pub fn status<P>(path: P, simple: bool) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let canonicalized_path = fs::canonicalize(path)?;
    for deploy_path in get_deploy_paths() {
        if deploy_path.to == canonicalized_path || deploy_path.from == canonicalized_path {
            let from_files = create_filemap(&deploy_path.from)?;
            let to_files = create_filemap(&deploy_path.to)?;

            let lookup = from_files
                .keys()
                .chain(to_files.keys())
                .collect::<BTreeSet<_>>()
                .iter()
                .map(|f| {
                    let from = from_files.get(*f);
                    let to = to_files.get(*f);
                    let s = get_status(from, to);
                    let ff = match s {
                        DeployStatus::Deployed => to,
                        DeployStatus::UnDeployed => from,
                        DeployStatus::UnManaged => to,
                        DeployStatus::Conflict => to,
                    }
                    .unwrap();

                    (
                        s,
                        ff.strip_prefix(path).unwrap_or(ff.as_path()).to_path_buf(),
                    )
                })
                .into_group_map();

            for s in vec![
                DeployStatus::Deployed,
                DeployStatus::UnManaged,
                DeployStatus::UnDeployed,
                DeployStatus::Conflict,
            ] {
                if lookup.contains_key(&s) {
                    if !simple {
                        print_status_description(&s);
                    }
                    print_status(&lookup, s)?;
                    if !simple {
                        println!("");
                    }
                }
            }

            return Ok(());
        }
    }
    return Err(anyhow::anyhow!(
        "{:?} is not managed directory.",
        canonicalized_path
    ));
}
