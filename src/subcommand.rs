//! Subcommand module
//!
//! This module contains subcommands.
//! Each subcommand is implemented as a function.
use crate::config::AppConfig;
use crate::config::Repository;
use crate::deploy_status::{get_status, DeployStatus};
use crate::fs;
use anyhow::{bail, Context as _, Ok, Result};
use itertools::Itertools;
use nu_ansi_term::Color::{Fixed, Green, Red, Yellow};
use std::fs::{read_dir, ReadDir};
use std::path::{Path, PathBuf};
use std::process::Command;

fn create_deploy_path<'a, P>(
    path: P,
    repo: &'a Repository,
) -> impl Iterator<Item = Result<(PathBuf, ReadDir, PathBuf)>> + 'a
where
    P: AsRef<Path> + 'a,
{
    let path = path.as_ref().to_path_buf();
    let repo_path = path.join(&repo.name);
    repo.deploy.iter().map(move |(from_dirname, to)| {
        let from_path = repo_path.join(from_dirname);
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

fn deploy_impl<P>(repo: &Repository, path: P, quiet: bool, force: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    log::trace!("deploy_impl({:?}, {:?}, {:?})", path.as_ref(), quiet, force);

    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, repo)
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
                    print_deploy_status(&DeployStatus::Deployed, &from, &to)?;
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
                        print_deploy_status(&DeployStatus::Deployed, &from, &to)?;
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
    repo_name: &Option<String>,
    quiet: bool,
    force: bool,
) -> Result<()> {
    log::trace!(
        "deploy({:?}, {:?}, {:?}, {:?})",
        app_config,
        repo_name,
        quiet,
        force
    );

    app_config
        .repos
        .iter()
        .filter(|repo| {
            // if repo is specified, skip other repo.
            if let Some(repo_name) = repo_name.as_ref() {
                repo.name == *repo_name
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, repo)| {
            let path = app_config.to_pathbuf()?;

            if !quiet {
                if index > 0 {
                    println!();
                }
                println!("Deploy {:}", repo.name);
            }

            // deploy
            deploy_impl(repo, path, quiet, force)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                log::error!("{:?}", e);
            }
        });
    Ok(())
}

fn undeploy_impl<P>(repo: &Repository, path: P, quiet: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    log::trace!("undeploy_impl({:?}, {:?})", path.as_ref(), quiet);

    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, repo)
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
                    print_deploy_status(&DeployStatus::UnDeployed, &from, &to)?;
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
pub fn undeploy(app_config: &AppConfig, repo_name: &Option<String>, quiet: bool) -> Result<()> {
    log::trace!("undeploy({:?}, {:?}, {:?})", app_config, repo_name, quiet);

    app_config
        .repos
        .iter()
        .filter(|repo| {
            // if repo is specified, skip other repo.
            if let Some(repo_name) = repo_name.as_ref() {
                repo.name == *repo_name
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, repo)| {
            let path = app_config.to_pathbuf()?;

            if !quiet {
                if index > 0 {
                    println!();
                }
                println!("UnDeploy {:}", repo.name);
            }

            // undeploy
            undeploy_impl(repo, path, quiet)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                log::error!("{:?}", e);
            }
        });
    Ok(())
}

fn print_deploy_status<P, Q>(status: &DeployStatus, from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    println!(
        "{:>12} {}",
        match status {
            DeployStatus::Deployed => Green
                .paint(format!("{:>12}", status.to_string()))
                .to_string(),
            DeployStatus::UnDeployed => Yellow
                .paint(format!("{:>12}", status.to_string()))
                .to_string(),
            DeployStatus::Conflict { .. } =>
                Red.paint(format!("{:>12}", status.to_string())).to_string(),
            DeployStatus::UnManaged => Fixed(8)
                .paint(format!("{:>12}", status.to_string()))
                .to_string(),
        },
        {
            let from_str = from.as_ref().to_string_lossy();
            let to_str = to.as_ref().to_string_lossy();

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
        }
    );
    Ok(())
}

fn status_impl<P>(repo: &Repository, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    log::trace!("status_impl({:?})", path.as_ref());

    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, repo)
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
                to_path.to_string_lossy()
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
                print_deploy_status(&status, from, to).expect("print error");
            }
        });

    Ok(())
}

pub fn status(app_config: &AppConfig, repo_name: &Option<String>) -> Result<()> {
    log::trace!("status({:?}, {:?})", app_config, repo_name);
    app_config
        .repos
        .iter()
        .filter(|repo| {
            // if repo is specified, skip other repo.
            if let Some(repo_name) = repo_name.as_ref() {
                repo.name == *repo_name
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, repo)| {
            let path = app_config.to_pathbuf()?;

            if index > 0 {
                println!();
            }
            println!("Repo {:}", repo.name);
            log::info!(
                "{:} => {:}",
                repo.url,
                path.join(&repo.name).to_string_lossy()
            );

            // TODO: repo自体のstatusを表示する
            // ディレクトリの存在チェック
            // gitの存在チェック
            // gitのstatusを表示する
            // gitのbranchを表示する
            // gitのremoteを表示する
            // gitのtagを表示する
            status_impl(repo, path)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                log::error!("{:?}", e);
            }
        });

    Ok(())
}

fn git_update(repo: &Repository, path: &Path, quiet: bool, verbose: bool) -> Result<()> {
    log::trace!("git_update({:?}, {:?})", repo, path);

    let path = path.join(&repo.name);

    // update git repository
    let mut git = Command::new("git");
    if path.exists() {
        git.arg("pull").current_dir(path);
    } else {
        git.arg("clone").arg(&repo.url).arg(path);
        if verbose {
            git.arg("-v");
        }
    }
    if quiet {
        git.arg("-q");
    }

    let output = git.output().with_context(|| {
        format!(
            "Failed to execute git {:}",
            git.get_args().map(|s| s.to_string_lossy()).join(" ")
        )
    })?;
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
pub fn update(
    app_config: &AppConfig,
    repo_name: &Option<String>,
    quiet: bool,
    verbose: bool,
    force: bool,
) -> Result<()> {
    log::trace!(
        "update({:?}, {:?}, {:?}, {:?}, {:?})",
        app_config,
        repo_name,
        quiet,
        verbose,
        force
    );

    app_config
        .repos
        .iter()
        .filter(|repo| {
            // if repo is specified, skip other repo.
            if let Some(repo_name) = repo_name.as_ref() {
                repo.name == *repo_name
            } else {
                true
            }
        })
        .enumerate()
        .map(|(index, repo)| {
            let path = app_config.to_pathbuf()?;

            if !quiet {
                if index > 0 {
                    println!();
                }
                println!("Update {:}", repo.name);
                println!(
                    "  {:} => {:}",
                    repo.url,
                    path.join(&repo.name).to_string_lossy()
                );
            }

            git_update(repo, &path, quiet, verbose)?;

            // deploy
            deploy_impl(repo, &path, quiet, force)?;

            Ok(())
        })
        .inspect(|r| {
            log::debug!("Update result: {:?}", r);
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
