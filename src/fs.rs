//! File system utilities.
use anyhow::Result;
use dunce::simplified;
use path_abs::PathAbs;
use std::path::Path;
use std::path::PathBuf;
use trash::delete;

#[cfg(any(target_os = "linux", target_os = "windows"))]
use anyhow::anyhow;

#[cfg(target_os = "linux")]
use std::fs;

#[cfg(target_os = "linux")]
use trash::Error;

pub fn absolutize<P>(path: P) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    Ok(simplified(PathAbs::new(path)?.as_path()).to_path_buf())
}

pub fn symlink<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    #[cfg(target_family = "unix")]
    {
        std::os::unix::fs::symlink(from, to)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let from = from.as_ref();
        if from.is_file() {
            std::os::windows::fs::symlink_file(from, to)?;
            Ok(())
        } else if from.is_dir() {
            std::os::windows::fs::symlink_dir(from, to)?;
            Ok(())
        } else {
            Err(anyhow!(
                "Can not deploy. {:?} is not file or directory.",
                from
            ))
        }
    }
}

pub fn remove<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    #[cfg(target_os = "linux")]
    {
        match delete(&path) {
            Ok(_) => Ok(()),
            Err(Error::FileSystem { .. }) => {
                fs::remove_file(&path)?;
                Ok(())
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        delete(&path)?;
        Ok(())
    }
}
