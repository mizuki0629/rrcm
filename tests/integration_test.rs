use ansi_term::Colour::{Green, Red, Yellow};
use anyhow::Result;
use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use indexmap::{indexmap, IndexMap};
use indoc::formatdoc;
use predicates::prelude::*;
use rrcm::config::AppConfig;
use rrcm::config::OsPath;
use rstest::rstest;
use std::fs;
use std::fs::OpenOptions;

fn create_temp_dir() -> Result<TempDir> {
    Ok(TempDir::new()?.into_persistent_if(false))
}

fn create_app_config(
    temp: &assert_fs::TempDir,
    repos: &IndexMap<String, String>,
) -> Result<ChildPath> {
    let tmpdir = temp.path().to_string_lossy();
    let config_file = temp.child("config.yaml");

    let dotfiles = OsPath {
        windows: Some(format!("{}\\dotfiles", tmpdir)),
        mac: Some(format!("{}/dotfiles", tmpdir)),
        linux: Some(format!("{}/dotfiles", tmpdir)),
    };

    let deploy = indexmap!(
        String::from("home") => OsPath {
            windows: Some(format!("{}\\home",tmpdir)),
            mac: Some(format!("{}/home",tmpdir)),
            linux: Some(format!("{}/home",tmpdir)),
        },
        String::from("config") => OsPath {
            windows: Some(format!("{}\\config",tmpdir)),
            mac: Some(format!("{}/config",tmpdir)),
            linux: Some(format!("{}/config",tmpdir)),
        },
        String::from("config_local") => OsPath {
            windows: Some(format!("{}\\config_local",tmpdir)),
            mac: Some(format!("{}/config_local",tmpdir)),
            linux: Some(format!("{}/config_local",tmpdir)),
        },
    );

    fs::create_dir(temp.path().join("home"))?;
    fs::create_dir(temp.path().join("config"))?;
    fs::create_dir(temp.path().join("config_local"))?;

    config_file.write_str(&serde_yaml::to_string(&AppConfig {
        dotfiles,
        deploy,
        repos: repos.clone(),
    })?)?;

    Ok(config_file)
}

fn create_cmd(
    config_file: &assert_fs::fixture::ChildPath,
    subcommand: &str,
    repo: &Option<String>,
    quiet: bool,
    verbose: bool,
    trace: bool,
    debug: bool,
) -> Result<Command> {
    let mut cmd = Command::cargo_bin("rrcm")?;
    if quiet {
        cmd.arg("--quiet");
    }
    if verbose {
        cmd.arg("--verbose");
    }
    if trace {
        cmd.arg("--trace");
    }
    if debug {
        cmd.arg("--debug");
    }
    cmd.arg("--config").arg(config_file.path());
    cmd.arg(subcommand);
    if let Some(repo) = repo {
        cmd.arg(repo);
    }
    Ok(cmd)
}

fn assert_symlink<P, Q>(path: P, target: Q) -> Result<()>
where
    P: AsRef<std::path::Path>,
    Q: AsRef<std::path::Path>,
{
    pretty_assertions::assert_eq!(
        true,
        path.as_ref().symlink_metadata()?.file_type().is_symlink()
    );
    pretty_assertions::assert_eq!(target.as_ref(), path.as_ref().read_link()?);
    Ok(())
}

#[rstest]
#[case(true, true, true, false, false)]
#[case(true, true, false, true, false)]
#[case(true, true, false, false, true)]
#[case(true, false, true, true, false)]
#[case(true, false, true, false, true)]
#[case(true, false, false, true, true)]
#[case(false, true, true, true, false)]
#[case(false, true, true, false, true)]
#[case(false, true, false, true, true)]
#[case(false, false, true, true, true)]
fn test_log_arg_error(
    #[case] is_short: bool,
    #[case] quiet: bool,
    #[case] verbose: bool,
    #[case] trace: bool,
    #[case] debug: bool,
) -> Result<()> {
    let mut cmd = Command::cargo_bin("rrcm")?;
    if quiet {
        if is_short {
            cmd.arg("-q");
        } else {
            cmd.arg("--quiet");
        }
    }
    if verbose {
        if is_short {
            cmd.arg("-v");
        } else {
            cmd.arg("--verbose");
        }
    }
    if trace {
        if is_short {
            cmd.arg("-t");
        } else {
            cmd.arg("--trace");
        }
    }
    if debug {
        if is_short {
            cmd.arg("-d");
        } else {
            cmd.arg("--debug");
        }
    }
    cmd.arg("status");
    cmd.assert().failure().stdout("");
    Ok(())
}

mod win_need_admin {

    use super::*;

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false, false, false)]
    #[case(None, false, false, false, false)]
    #[case(None, true, false, false, false)]
    #[case(None, false, true, false, false)]
    #[case(None, false, false, true, false)]
    #[case(None, false, false, false, true)]
    fn test_update(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] trace: bool,
        #[case] debug: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = indexmap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
        );
        let config_file = create_app_config(&temp, &repos)?;

        let deploy_files = vec![
            (
                temp.path().join("config_local").join("dir"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("config_local")
                    .join("dir"),
            ),
            (
                temp.path().join("home").join(".test.cfg"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("home")
                    .join(".test.cfg"),
            ),
        ];

        let repo_path = temp.path().join("dotfiles").join("rrcm-test");
        let repo_string = formatdoc!(
            r#"
            Update rrcm-test
              https://github.com/mizuki0629/rrcm-test.git => {repo_path}
            "#,
            repo_path = repo_path.to_string_lossy()
        );
        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose, trace, debug)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(path, _)| {
                        output.contains(&format!(
                            "{} {}",
                            Green.paint("    Deployed"),
                            path.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        temp.close()?;
        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false, false, false)]
    #[case(None, false, false, false, false)]
    #[case(None, true, false, false, false)]
    #[case(None, false, true, false, false)]
    #[case(None, false, false, true, false)]
    fn test_update_update(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] trace: bool,
        #[case] debug: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = indexmap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
        );
        let config_file = create_app_config(&temp, &repos)?;

        let deploy_files = vec![
            (
                temp.path().join("config_local").join("dir"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("config_local")
                    .join("dir"),
            ),
            (
                temp.path().join("home").join(".test.cfg"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("home")
                    .join(".test.cfg"),
            ),
        ];

        let repo_path = temp.path().join("dotfiles").join("rrcm-test");
        let repo_string = formatdoc!(
            r#"
            Update rrcm-test
              https://github.com/mizuki0629/rrcm-test.git => {repo_path}
            "#,
            repo_path = repo_path.to_string_lossy()
        );
        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose, trace, debug)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(path, _)| {
                        output.contains(&format!(
                            "{} {}",
                            Green.paint("    Deployed"),
                            path.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        let repo_string = formatdoc!(
            r#"
            Update rrcm-test
              https://github.com/mizuki0629/rrcm-test.git => {repo_path}
            "#,
            repo_path = repo_path.to_string_lossy()
        );
        // update pull
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose, trace, debug)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose || trace || debug {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string));
        } else {
            cmd.assert().success().stdout(repo_string);
        }

        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        temp.close()?;
        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false, false, false)]
    #[case(None, false, false, false, false)]
    #[case(None, true, false, false, false)]
    #[case(None, false, true, false, false)]
    #[case(None, false, false, true, false)]
    fn test_update_undeploy_status(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] trace: bool,
        #[case] debug: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = indexmap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
        );
        let config_file = create_app_config(&temp, &repos)?;

        let deploy_files = vec![
            (
                temp.path().join("config_local").join("dir"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("config_local")
                    .join("dir"),
            ),
            (
                temp.path().join("home").join(".test.cfg"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("home")
                    .join(".test.cfg"),
            ),
        ];

        let repo_path = temp.path().join("dotfiles").join("rrcm-test");
        let repo_string = formatdoc!(
            r#"
            Update rrcm-test
              https://github.com/mizuki0629/rrcm-test.git => {repo_path}
            "#,
            repo_path = repo_path.to_string_lossy()
        );

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose, trace, debug)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(path, _)| {
                        output.contains(&format!(
                            "{} {}",
                            Green.paint("    Deployed"),
                            path.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        // undeploy
        let repo_string = "UnDeploy rrcm-test\n";
        let mut cmd = create_cmd(
            &config_file,
            "undeploy",
            &repo,
            quiet,
            verbose,
            trace,
            debug,
        )?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(_, target)| {
                        output.contains(&format!(
                            "{} {}",
                            Yellow.paint("  UnDeployed"),
                            target.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, _) in &deploy_files {
            pretty_assertions::assert_eq!(false, path.exists());
        }

        // status
        let repo_string = "Repo rrcm-test\n";
        let mut cmd = create_cmd(&config_file, "status", &repo, quiet, verbose, trace, debug)?;
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(repo_string))
            .stdout(predicate::function(|output: &str| {
                deploy_files.iter().all(|(_, target)| {
                    output.contains(&format!(
                        "{} {}",
                        Yellow.paint("  UnDeployed"),
                        target.to_string_lossy()
                    ))
                })
            }));

        temp.close()?;
        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false, false, false,false)]
    #[case(None, false, false, false, false, false)]
    #[case(None, true, false, false, false, false)]
    #[case(None, false, true, false, false, false)]
    #[case(None, false, false, true, false, false)]
    #[case(None, false, false, false, true, false)]
    #[case(None, false, false, false, false, true)]
    fn test_update_undeploy_deploy_status(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] trace: bool,
        #[case] debug: bool,
        #[case] force: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = indexmap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
        );
        let config_file = create_app_config(&temp, &repos)?;

        let deploy_files = vec![
            (
                temp.path().join("config_local").join("dir"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("config_local")
                    .join("dir"),
            ),
            (
                temp.path().join("home").join(".test.cfg"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("home")
                    .join(".test.cfg"),
            ),
        ];

        let repo_path = temp.path().join("dotfiles").join("rrcm-test");
        let repo_string = formatdoc!(
            r#"
            Update rrcm-test
              https://github.com/mizuki0629/rrcm-test.git => {repo_path}
            "#,
            repo_path = repo_path.to_string_lossy()
        );

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose, trace, debug)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(path, _)| {
                        output.contains(&format!(
                            "{} {}",
                            Green.paint("    Deployed"),
                            path.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        // undeploy
        let repo_string = "UnDeploy rrcm-test\n";
        let mut cmd = create_cmd(
            &config_file,
            "undeploy",
            &repo,
            quiet,
            verbose,
            trace,
            debug,
        )?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(_, target)| {
                        output.contains(&format!(
                            "{} {}",
                            Yellow.paint("  UnDeployed"),
                            target.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, _) in &deploy_files {
            pretty_assertions::assert_eq!(false, path.exists());
        }

        // deploy
        let repo_string = "Deploy rrcm-test\n";
        let mut cmd = create_cmd(&config_file, "deploy", &repo, quiet, verbose, trace, debug)?;
        if force {
            cmd.arg("--force");
        }
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(path, _)| {
                        output.contains(&format!(
                            "{} {}",
                            Green.paint("    Deployed"),
                            path.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        // status
        let repo_string = "Repo rrcm-test\n";
        let mut cmd = create_cmd(&config_file, "status", &repo, quiet, verbose, trace, debug)?;
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(repo_string))
            .stdout(predicate::function(|output: &str| {
                deploy_files.iter().all(|(path, _)| {
                    output.contains(&format!(
                        "{} {}",
                        Green.paint("    Deployed"),
                        path.to_string_lossy()
                    ))
                })
            }));

        temp.close()?;

        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false, false, false,false)]
    #[case(None, false, false, false, false, false)]
    #[case(None, true, false, false, false, false)]
    #[case(None, false, true, false, false, false)]
    #[case(None, false, false, true, false, false)]
    #[case(None, false, false, false, true, false)]
    #[case(None, false, false, false, false, true)]
    fn test_update_deploy(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] trace: bool,
        #[case] debug: bool,
        #[case] force: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = indexmap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
        );
        let config_file = create_app_config(&temp, &repos)?;

        let deploy_files = vec![
            (
                temp.path().join("config_local").join("dir"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("config_local")
                    .join("dir"),
            ),
            (
                temp.path().join("home").join(".test.cfg"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("home")
                    .join(".test.cfg"),
            ),
        ];

        let repo_path = temp.path().join("dotfiles").join("rrcm-test");
        let repo_string = formatdoc!(
            r#"
            Update rrcm-test
              https://github.com/mizuki0629/rrcm-test.git => {repo_path}
            "#,
            repo_path = repo_path.to_string_lossy()
        );

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose, trace, debug)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string))
                .stdout(predicate::function(|output: &str| {
                    deploy_files.iter().all(|(path, _)| {
                        output.contains(&format!(
                            "{} {}",
                            Green.paint("    Deployed"),
                            path.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        let repo_string = "Deploy rrcm-test\n";
        // deploy
        let mut cmd = create_cmd(&config_file, "deploy", &repo, quiet, verbose, trace, debug)?;
        if force {
            cmd.arg("--force");
        }
        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose || trace || debug {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(repo_string));
        } else {
            cmd.assert().success().stdout(repo_string);
        }

        for (path, target) in &deploy_files {
            assert_symlink(path, target)?;
        }

        temp.close()?;

        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false, false, false)]
    #[case(None, false, false, false, false)]
    #[case(None, true, false, false, false)]
    #[case(None, false, true, false, false)]
    #[case(None, false, false, true, false)]
    fn test_deploy_error(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] trace: bool,
        #[case] debug: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = indexmap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
        );
        let config_file = create_app_config(&temp, &repos)?;

        let deploy_files = vec![
            (
                temp.path().join("config_local").join("dir"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("config_local")
                    .join("dir"),
            ),
            (
                temp.path().join("home").join(".test.cfg"),
                temp.path()
                    .join("dotfiles")
                    .join("rrcm-test")
                    .join("home")
                    .join(".test.cfg"),
            ),
        ];

        let repo_path = temp.path().join("dotfiles").join("rrcm-test");
        let repo_string = formatdoc!(
            r#"
            Update rrcm-test
              https://github.com/mizuki0629/rrcm-test.git => {repo_path}
            "#,
            repo_path = repo_path.to_string_lossy()
        );

        for (path, _) in &deploy_files {
            OpenOptions::new().write(true).create(true).open(path)?;
        }

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose, trace, debug)?;
        if quiet {
            cmd.assert()
                .success()
                .stdout("")
                .stderr(predicate::function(|output: &str| {
                    deploy_files.iter().any(|(path, target)| {
                        output.contains(&format!(
                            "Failed to deploy {} -> {}",
                            target.to_string_lossy(),
                            path.to_string_lossy()
                        ))
                    })
                }));
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::contains(&repo_string))
                .stderr(predicate::function(|output: &str| {
                    deploy_files.iter().any(|(path, target)| {
                        output.contains(&format!(
                            "Failed to deploy {} -> {}",
                            target.to_string_lossy(),
                            path.to_string_lossy()
                        ))
                    })
                }));
        }
        for (path, _) in &deploy_files {
            pretty_assertions::assert_eq!(path.is_symlink(), false);
        }

        let repo_string = "Repo rrcm-test\n";
        // status
        let mut cmd = create_cmd(&config_file, "status", &repo, quiet, verbose, trace, debug)?;
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(repo_string))
            .stdout(predicate::function(|output: &str| {
                deploy_files.iter().all(|(path, _)| {
                    output.contains(&format!(
                        "{} {}",
                        Red.paint("    Conflict"),
                        path.to_string_lossy()
                    ))
                })
            }));

        temp.close()?;
        Ok(())
    }
}
