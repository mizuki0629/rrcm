use anyhow;
use dirs;
use std::collections::HashMap;
use std::collections::HashSet;
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

fn is_managed(src: Option<&DirEntry>, dst: Option<&DirEntry>) -> bool {
    let mut is_managed = false;
    if let Some(src_entry) = src {
        if let Some(dst_entry) = dst {
            if let Ok(file_type) = dst_entry.file_type() {
                if file_type.is_symlink() {
                    is_managed = src_entry.path() == fs::read_link(dst_entry.path()).unwrap();
                }
            }
        }
    }
    is_managed
}

#[cfg(target_family = "unix")]
fn deploy(src: &Path, dst: &Path) -> anyhow::Result<()> {
    let mut link = PathBuf::new();
    link.push(dst);
    link.set_file_name(src.file_name().unwrap());
    std::os::unix::fs::symlink(src, link)?;
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
        let is_managed = is_managed(src, dst);
        println!(
            "{:} {:?} {:?}",
            if is_managed { "*" } else { " " },
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
