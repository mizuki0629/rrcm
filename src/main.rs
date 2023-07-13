use anyhow;
use dirs;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::fs::DirEntry;
use std::path;
use std::path::Path;
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

#[derive(Debug,PartialEq,Eq)]
enum Status {
    Managed,
    Deployed,
    Conflict,
    UnManaged,
}

fn status(src: Option<&DirEntry>, dst: Option<&DirEntry>) -> Status {
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
            s = Status::Managed;
        }
    }
    s
}

fn manage(src: &Path, dst_dir: &Path) -> anyhow::Result<()> {
    let dst = Path::new(dst_dir).join(src.file_name().unwrap());
    std::fs::rename(src, dst)?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn deploy(src: &Path, dst_dir: &Path) -> anyhow::Result<()> {
    let dst = Path::new(dst_dir).join(src.file_name().unwrap());
    std::os::unix::fs::symlink(src, dst)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut path = dirs::home_dir().unwrap();
    path.push("dotfiles");
    path.push("config");

    let dotfiles = create_filemap(&path)?;

    let config_dir_files = create_filemap(&dirs::config_dir().unwrap())?;

    let a = dotfiles.keys().collect::<HashSet<_>>();
    let b = config_dir_files.keys().collect::<HashSet<_>>();
    let filelist = a.union(&b).collect::<Vec<_>>();

    println!("config");
    for key in filelist {
        let src = dotfiles.get(*key);
        let dst = config_dir_files.get(*key);
        let s = status(src, dst);
        println!(
            "{:?} {:?} {:?}",
            s,
            if let Some(entry) = src {
                entry.file_name()
            } else {
                OsString::new()
            },
            if let Some(entry) = dst {
                entry.file_name()
            } else {
                OsString::new()
            },
        );
    }

    anyhow::Ok(())
}
