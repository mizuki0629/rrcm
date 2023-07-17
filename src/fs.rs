use anyhow;
use dunce::simplified;
use path_abs;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use trash;

pub fn canonicalize<P>(path: P) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path>,
{
    return Ok(simplified(fs::canonicalize(path)?.as_path()).to_path_buf());
}

pub fn absolutize<P>(path: P) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path>,
{
    return Ok(simplified(path_abs::PathAbs::new(path)?.as_path()).to_path_buf());
}

pub fn symlink<P, Q>(from: P, to: Q) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    #[cfg(target_family = "unix")]
    {
        std::os::unix::fs::symlink(from, to)?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let from = from.as_ref();
        if from.is_file() {
            std::os::windows::fs::symlink_file(from, to)?;
            return Ok(());
        } else if from.is_dir() {
            std::os::windows::fs::symlink_dir(from, to)?;
            return Ok(());
        } else {
            return Err(anyhow::anyhow!(
                "Can not deploy. {:?} is not file or directory.",
                from
            ));
        }
    }
}

pub fn remove<P>(path: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    trash::delete(path)?;
    Ok(())
}
