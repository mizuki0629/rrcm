//! Subcommand module
//!
//! This module contains subcommands.
//! Each subcommand is implemented as a function.
use crate::config::AppConfig;
use crate::deploy_status::{get_status, DeployStatus};
use crate::fs;
use crate::path::strip_home;
use anyhow::{bail, Context as _, Ok, Result};
use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use itertools::Itertools;
use std::fs::{read_dir, ReadDir};
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::Command;

fn create_deploy_path<'a, P>(
    path: P,
    app_config: &'a AppConfig,
) -> impl Iterator<Item = Result<(PathBuf, ReadDir, PathBuf)>> + 'a
where
    P: AsRef<Path> + 'a,
{
    let path = path.as_ref().to_path_buf();
    app_config.deploy.iter().map(move |(from_dirname, to)| {
        let from_path = path.join(PathBuf::from(from_dirname));
        let from_readdir = read_dir(&from_path).with_context(|| {
            format!(
                "Failed to read deploy source directory {:}",
                from_path.to_string_lossy()
            )
        })?;
        let to_path = to.to_pathbuf().with_context(|| {
            format!(
                "Failed to read deploy destination directory \"{:}\"",
                from_dirname
            )
        })?;
        Ok((from_path, from_readdir, to_path))
    })
}

fn create_deploy_status(
    deploy_status_list: Vec<(PathBuf, ReadDir, PathBuf)>,
) -> impl Iterator<Item = Result<(DeployStatus, PathBuf, PathBuf)>> {
    deploy_status_list
        .into_iter()
        .flat_map(|(from_path, from_readdir, to_path)| {
            from_readdir.map(move |entry| {
                let from = entry
                    .with_context(|| {
                        format!(
                            "Failed to read deploy source directory entry {:}",
                            from_path.to_string_lossy()
                        )
                    })?
                    .path();

                let to = to_path.join(from.file_name().with_context(|| {
                    format!("Failed to get file name from {:}", from.to_string_lossy())
                })?);

                Ok((get_status(&from, &to), from, to))
            })
        })
}

fn deploy_impl<P>(app_config: &AppConfig, path: P, quiet: bool, force: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    log::trace!("deploy_impl({:?}, {:?}, {:?})", path.as_ref(), quiet, force);

    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, app_config)
        .inspect(|r| {
            log::debug!("Deploy path: {:?}", r);
        })
        .filter_map(Result::ok)
        .collect();

    create_deploy_status(deploy_paths)
        .inspect(|r| {
            log::debug!("Deploy status: {:?}", r);
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(status, from, to)| match status {
            DeployStatus::UnDeployed => {
                fs::symlink(&from, &to).with_context(|| {
                    format!(
                        "Failed to create symlink {:} -> {:}",
                        from.to_string_lossy(),
                        to.to_string_lossy()
                    )
                })?;

                if !quiet {
                    print_deploy_status(path, &DeployStatus::Deployed, &from, &to)?;
                }
                Ok((from, to))
            }
            DeployStatus::Deployed => Ok((from, to)),
            DeployStatus::Conflict { cause } => {
                if force {
                    fs::remove(&to).with_context(|| {
                        format!("Failed to remove file {:}", to.to_string_lossy())
                    })?;

                    fs::symlink(&from, &to).with_context(|| {
                        format!(
                            "Failed to create symlink {:} -> {:}",
                            from.to_string_lossy(),
                            to.to_string_lossy()
                        )
                    })?;

                    if !quiet {
                        print_deploy_status(path, &DeployStatus::Deployed, &from, &to)?;
                    }
                    return Ok((from, to));
                }

                Err(anyhow::anyhow!("{:}", cause).context(format!(
                    "Failed to deploy {:} -> {:}",
                    from.to_string_lossy(),
                    to.to_string_lossy()
                )))
            }
            DeployStatus::UnManaged => {
                bail!("File not exists {:}", to.to_string_lossy());
            }
        })
        .inspect(|r| {
            log::debug!("Deploy result: {:?}", r);
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

pub fn deploy(
    app_config: &AppConfig,
    repo: Option<String>,
    quiet: bool,
    force: bool,
) -> Result<()> {
    log::trace!(
        "deploy({:?}, {:?}, {:?}, {:?})",
        app_config,
        repo,
        quiet,
        force
    );

    app_config
        .repos
        .iter()
        .filter(|(name, _)| {
            // if repo is specified, skip other repo.
            if let Some(repo) = repo.as_ref() {
                name == &repo
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, (name, _))| {
            let path = app_config.to_pathbuf()?.join(name);

            if !quiet {
                if index > 0 {
                    println!();
                }
                println!("Deploy {:}", name);
            }

            // deploy
            deploy_impl(app_config, path, quiet, force)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                log::error!("{:?}", e);
            }
        });
    Ok(())
}

fn undeploy_impl<P>(app_config: &AppConfig, path: P, quiet: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    log::trace!("undeploy_impl({:?}, {:?})", path.as_ref(), quiet);

    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, app_config)
        .inspect(|r| {
            log::debug!("Deploy path: {:?}", r);
        })
        .filter_map(Result::ok)
        .collect();

    create_deploy_status(deploy_paths)
        .inspect(|r| {
            log::debug!("Deploy status: {:?}", r);
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(status, from, to)| match status {
            DeployStatus::UnDeployed => Ok((from, to)),
            DeployStatus::Deployed => {
                fs::remove(&to)
                    .with_context(|| format!("Failed to remove file {:}", to.to_string_lossy()))?;

                if !quiet {
                    print_deploy_status(path, &DeployStatus::UnDeployed, &from, &to)?;
                }
                Ok((from, to))
            }
            DeployStatus::Conflict { .. } => Ok((from, to)),
            DeployStatus::UnManaged => {
                bail!("File not exists {:}", to.to_string_lossy());
            }
        })
        .inspect(|r| {
            log::debug!("Undeploy result: {:?}", r);
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

/// undeploy files
/// # Arguments
/// * `repo` - repo name
/// * `quiet` - quiet mode
/// # Example
/// ```no_run
/// use rrcm::undeploy;
/// use rrcm::config::load_app_config;
/// let app_config = load_app_config().unwrap();
/// undeploy(&app_config, None, false);
/// ```
///     
pub fn undeploy(app_config: &AppConfig, repo: Option<String>, quiet: bool) -> Result<()> {
    log::trace!("undeploy({:?}, {:?}, {:?})", app_config, repo, quiet);

    app_config
        .repos
        .iter()
        .filter(|(name, _)| {
            // if repo is specified, skip other repo.
            if let Some(repo) = repo.as_ref() {
                name == &repo
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, (name, _))| {
            let path = app_config.to_pathbuf()?.join(name);

            if !quiet {
                if index > 0 {
                    println!();
                }
                println!("UnDeploy {:}", name);
            }

            // undeploy
            undeploy_impl(app_config, path, quiet)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                log::error!("{:?}", e);
            }
        });
    Ok(())
}

fn print_deploy_status<P, Q, R>(path: P, status: &DeployStatus, from: Q, to: R) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    R: AsRef<Path>,
{
    stdout()
        .execute(match status {
            DeployStatus::Deployed => SetForegroundColor(Color::Green),
            DeployStatus::UnDeployed => SetForegroundColor(Color::Yellow),
            DeployStatus::Conflict { .. } => SetForegroundColor(Color::Red),
            DeployStatus::UnManaged => SetForegroundColor(Color::Grey),
        })?
        .execute(Print(format!("{:>12}", format!("{:}", status))))?
        .execute(ResetColor)?
        .execute(Print(format!(" {}\n", {
            let from_str = from.as_ref().strip_prefix(path)?.to_string_lossy();
            let to = strip_home(&to);
            let to_str = to.to_string_lossy();

            match &status {
                DeployStatus::Deployed => {
                    format!("{:<20}", to_str)
                }
                DeployStatus::UnDeployed => format!("{:}", from_str),
                DeployStatus::UnManaged => format!("{:}", to_str),
                DeployStatus::Conflict { cause } => {
                    format!("{:<20} {:}", to_str, cause,)
                }
            }
        })))?;
    Ok(())
}

fn status_impl<P>(app_config: &AppConfig, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    log::trace!("status_impl({:?})", path.as_ref());

    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, app_config)
        .inspect(|result| {
            if let Err(e) = result {
                log::warn!("{:?}", e);
            }
        })
        .filter_map(Result::ok)
        .collect_vec();

    if log::log_enabled!(log::Level::Info) {
        log::info!("Deploy From => To ");
        for (from_path, _, to_path) in &deploy_paths {
            log::info!(
                "{:} => {:}",
                from_path.strip_prefix(path).unwrap().to_string_lossy(),
                strip_home(to_path).to_string_lossy()
            );
        }
    }

    create_deploy_status(deploy_paths)
        .inspect(|result| {
            if let Err(e) = result {
                log::warn!("{:?}", e);
            }
        })
        .filter_map(Result::ok)
        .for_each(|(status, from, to)| {
            if matches!(
                status,
                DeployStatus::Deployed | DeployStatus::UnDeployed | DeployStatus::Conflict { .. }
            ) {
                print_deploy_status(path, &status, from, to).expect("print error");
            }
        });

    Ok(())
}

pub fn status(app_config: &AppConfig, repo: Option<String>) -> Result<()> {
    log::trace!("status({:?}, {:?})", app_config, repo);
    app_config
        .repos
        .iter()
        .filter(|(name, _)| {
            // if repo is specified, skip other repo.
            if let Some(repo) = repo.as_ref() {
                name == &repo
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, (name, url))| {
            let path = app_config.to_pathbuf()?.join(name);

            if index > 0 {
                println!();
            }
            println!("Repo {:}", name);
            log::info!("{:} => {:}", url, path.to_string_lossy());

            status_impl(app_config, path)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                log::error!("{:?}", e);
            }
        });

    Ok(())
}

/// Update dotfiles from git repository and deploy.
/// If repo is specified, update only specified repo.
/// If repo is not specified, update all repos.
///
/// # Arguments
/// * `repo` - repository name
/// * `quiet` - quiet mode
/// * `verbose` - verbose mode
///
/// # Example
/// ```no_run
/// use rrcm::update;
/// use rrcm::config::load_app_config;
/// let app_config = load_app_config().unwrap();
/// update(&app_config, None, false, false);
/// ```
/// ```no_run
/// use rrcm::update;
/// use rrcm::config::load_app_config;
/// let app_config = load_app_config().unwrap();
/// update(&app_config, Some("repo".to_string()), false, false);
/// ```
pub fn update(
    app_config: &AppConfig,
    repo: Option<String>,
    quiet: bool,
    verbose: bool,
) -> Result<()> {
    log::trace!(
        "update({:?}, {:?}, {:?}, {:?})",
        app_config,
        repo,
        quiet,
        verbose
    );

    app_config
        .repos
        .iter()
        .filter(|(name, _)| {
            // if repo is specified, skip other repo.
            if let Some(repo) = repo.as_ref() {
                name == &repo
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, (name, url))| {
            let path = app_config.to_pathbuf()?.join(name);

            if !quiet {
                if index > 0 {
                    println!();
                }
                println!("Update {:}", name);
                println!("  {:} => {:}", url, path.to_string_lossy());
            }

            // update git repository
            let mut git = Command::new("git");
            if path.exists() {
                git.arg("pull").current_dir(&path);
            } else {
                git.arg("clone").arg(url).arg(&path);
                if verbose {
                    git.arg("-v");
                }
            }
            if quiet {
                git.arg("-q");
            }

            let output = git.output()?;
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr = stderr.trim_end();
            if !output.status.success() {
                bail!("{}", stderr);
            }

            if !quiet && !stderr.is_empty() {
                println!("{}", stderr);
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stdout = stdout.trim_end();
            if !stdout.is_empty() {
                log::info!("{}", stdout);
            }

            // deploy
            deploy_impl(app_config, &path, quiet, false)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                log::error!("{:?}", e);
            }
        });
    Ok(())
}
