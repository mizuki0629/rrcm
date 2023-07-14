use anyhow;
use clap::{Parser, Subcommand};
use color_print::cprintln;
use dirs;
use itertools::Itertools;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use std::path;
use std::path::Path;
use std::path::PathBuf;

// 存在するか       Y Y Y N
// Link             Y Y N /
// Link先が正しいか Y N / /
// ------------------------
//                  T F F F

fn create_filemap(path: &path::Path) -> anyhow::Result<HashMap<OsString, DirEntry>> {
    let mut dst: HashMap<_, _> = HashMap::new();
    for entry in fs::read_dir(path)?.filter_map(|e| e.ok()) {
        dst.insert(entry.file_name(), entry);
    }
    anyhow::Ok(dst)
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Status {
    UnDeployed,
    Deployed,
    Conflict,
    UnManaged,
}

fn get_status(src: Option<&DirEntry>, dst: Option<&DirEntry>) -> Status {
    let mut s = Status::UnManaged;
    if let Some(src_entry) = src {
        if let Some(dst_entry) = dst {
            s = Status::Conflict;
            if let Ok(file_type) = dst_entry.file_type() {
                if file_type.is_symlink() {
                    if src_entry.path() == fs::read_link(dst_entry.path()).unwrap() {
                        s = Status::Deployed;
                    }
                }
            }
        } else {
            s = Status::UnDeployed;
        }
    }
    s
}

#[cfg(target_family = "unix")]
fn symlink<P, Q>(from: P, to: Q) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    std::os::unix::fs::symlink(from, to)?;
    return Ok(());
}

fn deploy<P>(path: P, targets: Vec<Target>) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let canonicalized_path = path.canonicalize()?;
    for target in targets {
        if target.from == canonicalized_path.parent().unwrap() {
            std::os::unix::fs::symlink(path, target.to.join(path.file_name().unwrap()))?;
            return Ok(());
        }
    }
    return Err(anyhow::anyhow!(
        "{:?} is not managed directory. Please add config.",
        canonicalized_path
    ));
}

fn add<P>(path: P, targets: Vec<Target>) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let canonicalized_path = path.canonicalize()?;
    for target in targets {
        if target.to == canonicalized_path.parent().unwrap() {
            let manage_path = target.from.join(path.file_name().unwrap());
            if manage_path.exists() {
                return Err(anyhow::anyhow!("{:?} is already exists.", manage_path));
            }
            std::fs::rename(path, manage_path.clone())?;
            symlink(manage_path, path)?;
            return Ok(());
        }
    }
    return Err(anyhow::anyhow!(
        "{:?} is not managed directory. Please add config.",
        canonicalized_path
    ));
}

struct Target {
    from: PathBuf,
    to: PathBuf,
}

fn get_targets() -> Vec<Target> {
    let mut targets = Vec::new();

    let mut from = dirs::home_dir().unwrap();
    from.push("dotfiles");
    from.push("config");

    targets.push(Target {
        from,
        to: dirs::config_dir().unwrap(),
    });

    let mut from = dirs::home_dir().unwrap();
    from.push("dotfiles");
    from.push("home");
    targets.push(Target {
        from,
        to: dirs::home_dir().unwrap(),
    });

    targets
}

fn list() -> anyhow::Result<()> {
    println!("Status From To");
    for target in get_targets() {
        let from_files = create_filemap(&target.from)?;
        let to_files = create_filemap(&target.to)?;

        let filelist = from_files
            .keys()
            .chain(to_files.keys())
            .collect::<BTreeSet<_>>();

        println!("-- {:?} {:?}", target.from, target.to);
        for key in filelist {
            let from = from_files.get(key);
            let to = to_files.get(key);
            let status = get_status(from, to);

            println!(
                "{:?} {:?} {:?}",
                status,
                from.map_or(OsString::new(), |v| v.file_name()),
                to.map_or(OsString::new(), |v| v.file_name()),
            );
        }
    }
    anyhow::Ok(())
}

fn print_status(
    path: &Path,
    lookup: &HashMap<Status, Vec<(Option<&DirEntry>, Option<&DirEntry>)>>,
    s: Status,
) -> anyhow::Result<()> {
    if let Some(l) = lookup.get(&s) {
        println!("");
        match s {
            Status::Deployed => {
            },
            Status::UnDeployed => {
                println!("Files are not deployed:");
                println!("  (use \"rrcm deploy <file>...\" to deploy files)");
            }
            Status::UnManaged => {
                println!("  (use \"rrcm add <file>...\" to manage and deploy files)");
            },
            Status::Conflict => {
                println!("  (symlink is not to managed file or not symlink.)");
                println!("  (you can move or delete file or )");
                println!("  (use \"rrcm deploy -f <file>...\" force deploy files)");
            },
        }

        for f in l {
            let ff = match s {
                Status::Deployed => f.1,
                Status::UnDeployed => f.0,
                Status::UnManaged => f.1,
                Status::Conflict => f.1,
            }
            .unwrap()
            .path();
            let ff = ff.strip_prefix(path).unwrap_or(ff.as_path()).to_str().unwrap();
            match s {
                Status::Deployed => cprintln!("    <green>{:?}: {:}</>", s, ff),
                Status::UnDeployed => cprintln!("    <yellow>{:?}: {:}</>", s, ff),
                Status::UnManaged => cprintln!("    <dim>{:?}: {:}</>", s, ff),
                Status::Conflict => cprintln!("    <red>{:?}: {:}</>", s, ff),
            };
        }
    }

    return Ok(());
}

fn status<P>(path: P, targets: Vec<Target>) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let canonicalized_path = path.canonicalize()?;
    for target in targets {
        if target.to == canonicalized_path {
            let from_files = create_filemap(&target.from)?;
            let to_files = create_filemap(&target.to)?;

            let filelist = from_files
                .keys()
                .chain(to_files.keys())
                .collect::<BTreeSet<_>>();

            let lookup = filelist
                .iter()
                .map(|f| {
                    let from = from_files.get(*f);
                    let to = to_files.get(*f);
                    (get_status(from, to), (from, to))
                })
                .into_group_map();

            print_status(path, &lookup, Status::Deployed)?;
            print_status(path, &lookup, Status::Conflict)?;
            print_status(path, &lookup, Status::UnManaged)?;
            print_status(path, &lookup, Status::UnDeployed)?;

            return Ok(());
        }
    }
    return Err(anyhow::anyhow!(
        "{:?} is not managed directory.",
        canonicalized_path
    ));
}

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
struct Args {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    Status {
        #[clap(default_value = ".")]
        path: PathBuf,
    },
    /// Print
    List,
    /// Add
    Add {
        /// file or directory path
        #[clap(required = true, ignore_case = true)]
        path: PathBuf,
    },
    /// Deploy config file
    Deploy,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.subcommand {
        SubCommands::Status { path } => {
            let targets = get_targets();
            status(path.as_path(), targets)
        }
        SubCommands::List => list(),
        SubCommands::Add { path } => {
            let targets = get_targets();
            add(path.as_path(), targets)
        }
        SubCommands::Deploy => {
            // let targets = get_targets();
            // let path = Path::new("/home/mizuki/dotfiles/config/rrcm");
            // deploy(path, targets)?;
            println!("{:?}", args.subcommand);
            anyhow::Ok(())
        }
    }
}
