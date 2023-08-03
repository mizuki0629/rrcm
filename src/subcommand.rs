//! Subcommand module
//!
//! This module contains subcommands.
//! Each subcommand is implemented as a function.
use crate::config::{self, AppConfig};
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
use std::io::{stderr, stdout};
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

pub fn print_error(e: &anyhow::Error) {
    let imp = |e| {
        stderr()
            .execute(SetForegroundColor(Color::Red))?
            .execute(Print("Error: "))?
            .execute(ResetColor)?
            .execute(Print(format!("{:?}\n", e)))?;
        Ok(())
    };
    imp(e).expect("Failed to print error");
}

pub fn print_warn(e: &anyhow::Error) {
    let imp = |e| {
        stderr()
            .execute(SetForegroundColor(Color::Yellow))?
            .execute(Print("Warn: "))?
            .execute(ResetColor)?
            .execute(Print(format!("{:?}\n", e)))?;
        Ok(())
    };
    imp(e).expect("Failed to print error");
}

fn deploy_impl<P>(path: P, quiet: bool, force: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, &config::load_app_config()?)
        .filter_map(Result::ok)
        .collect();

    create_deploy_status(deploy_paths)
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
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

pub fn deploy(repo: Option<String>, quiet: bool, force: bool) -> Result<()> {
    let app_config = config::load_app_config()?;
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
            deploy_impl(path, quiet, force)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                print_error(&e);
            }
        });
    Ok(())
}

fn undeploy_impl<P>(path: P, quiet: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, &config::load_app_config()?)
        .filter_map(Result::ok)
        .collect();

    create_deploy_status(deploy_paths)
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
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

pub fn undeploy(repo: Option<String>, quiet: bool) -> Result<()> {
    let app_config = config::load_app_config()?;
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
            undeploy_impl(path, quiet)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                print_error(&e);
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

fn status_impl<P>(app_config: &AppConfig, path: P, verbose: bool) -> Result<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let deploy_paths = create_deploy_path(path, app_config)
        .inspect(|result| {
            if verbose {
                if let Err(e) = result {
                    print_warn(e);
                }
            }
        })
        .filter_map(Result::ok)
        .collect_vec();

    if verbose {
        println!("Deploy From => To ");
        for (from_path, _, to_path) in &deploy_paths {
            println!(
                "    {:<20} => {:<20}",
                from_path.strip_prefix(path).unwrap().to_string_lossy(),
                strip_home(to_path).to_string_lossy()
            );
        }
        println!();
    }

    create_deploy_status(deploy_paths)
        .inspect(|result| {
            if verbose {
                if let Err(e) = result {
                    print_warn(e);
                }
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

pub fn status(repo: Option<String>, verbose: bool) -> Result<()> {
    let app_config = config::load_app_config()?;
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
            println!("  {:} => {:}", url, path.to_string_lossy());

            status_impl(&app_config, path, verbose)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                print_error(&e);
            }
        });

    Ok(())
}

fn add_git_option(git: &mut Command, quiet: bool, verbose: bool) {
    if quiet {
        git.arg("-q");
    }
    if verbose {
        git.arg("-v");
    }
}

pub fn update(repo: Option<String>, quiet: bool, verbose: bool) -> Result<()> {
    let app_config = config::load_app_config()?;
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
                add_git_option(&mut git, quiet, verbose);

                let status = git.status();
                if status.is_err() || !status.unwrap().success() {
                    bail!("Failed to pull {:}", url);
                }
            } else {
                git.arg("clone").arg(url).arg(&path);
                add_git_option(&mut git, quiet, verbose);

                let status = git.status();
                if status.is_err() || !status.unwrap().success() {
                    bail!("Failed to clone {:}", url);
                }
            }

            // deploy
            deploy_impl(&path, quiet, false)?;

            Ok(())
        })
        .for_each(|result| {
            if let Err(e) = result {
                print_error(&e);
            }
        });
    Ok(())
}
