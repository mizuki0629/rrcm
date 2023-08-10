use ansi_term::Colour::{Green, Red, Yellow};
use anyhow::Result;
use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use maplit::btreemap;
use predicates::prelude::*;
use rrcm::config::AppConfig;
use rrcm::config::OsPath;
use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;

fn create_temp_dir() -> Result<TempDir> {
    Ok(TempDir::new()?.into_persistent_if(false))
}

fn create_app_config(
    temp: &assert_fs::TempDir,
    repos: &BTreeMap<String, String>,
) -> Result<ChildPath> {
    let tmpdir = temp.path().to_string_lossy();
    let config_file = temp.child("config.toml");

    let dotfiles = OsPath {
        windows: Some(format!("{}\\dotfiles", tmpdir)),
        mac: Some(format!("{}/dotfiles", tmpdir)),
        linux: Some(format!("{}/dotfiles", tmpdir)),
    };

    let deploy = btreemap!(
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

    config_file.write_str(&toml::to_string(&AppConfig {
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
) -> Result<Command> {
    let mut cmd = Command::cargo_bin("rrcm")?;
    if quiet {
        cmd.arg("--quiet");
    }
    if verbose {
        cmd.arg("--verbose");
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

mod win_need_admin {

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false)]
    #[case(None, false, false)]
    #[case(None, true, false)]
    #[case(None, false, true)]
    fn test_update(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = btreemap!(
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
        let repo_string = format!(
            r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {repo_path}
Cloning into '{repo_path}'...
"#,
            repo_path = repo_path.to_string_lossy()
        );
        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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
    #[case(Some("rrcm-test".to_owned()), false, false)]
    #[case(None, false, false)]
    #[case(None, true, false)]
    #[case(None, false, true)]
    fn test_update_update(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = btreemap!(
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
        let repo_string = format!(
            r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {repo_path}
Cloning into '{repo_path}'...
"#,
            repo_path = repo_path.to_string_lossy()
        );
        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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

        let repo_string = format!(
            r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {repo_path}
"#,
            repo_path = repo_path.to_string_lossy()
        );
        // update pull
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string));
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
    #[case(Some("rrcm-test".to_owned()), false, false)]
    #[case(None, false, false)]
    #[case(None, true, false)]
    #[case(None, false, true)]
    fn test_update_undeploy_status(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = btreemap!(
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
        let repo_string = format!(
            r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {repo_path}
Cloning into '{repo_path}'...
"#,
            repo_path = repo_path.to_string_lossy()
        );

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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
        let mut cmd = create_cmd(&config_file, "undeploy", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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
        let mut cmd = create_cmd(&config_file, "status", &repo, quiet, verbose)?;
        cmd.assert()
            .success()
            .stdout(predicate::str::starts_with(repo_string))
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
    #[case(Some("rrcm-test".to_owned()), false, false, false)]
    #[case(None, false, false, false)]
    #[case(None, true, false, false)]
    #[case(None, false, true, false)]
    #[case(None, false, false, true)]
    fn test_update_undeploy_deploy_status(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] force: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = btreemap!(
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
        let repo_string = format!(
            r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {repo_path}
Cloning into '{repo_path}'...
"#,
            repo_path = repo_path.to_string_lossy()
        );

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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
        let mut cmd = create_cmd(&config_file, "undeploy", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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
        let mut cmd = create_cmd(&config_file, "deploy", &repo, quiet, verbose)?;
        if force {
            cmd.arg("--force");
        }
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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
        let mut cmd = create_cmd(&config_file, "status", &repo, quiet, verbose)?;
        cmd.assert()
            .success()
            .stdout(predicate::str::starts_with(repo_string))
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
    #[case(Some("rrcm-test".to_owned()), false, false, false)]
    #[case(None, false, false, false)]
    #[case(None, true, false, false)]
    #[case(None, false, true, false)]
    #[case(None, false, false, true)]
    fn test_update_deploy(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] force: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = btreemap!(
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
        let repo_string = format!(
            r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {repo_path}
Cloning into '{repo_path}'...
"#,
            repo_path = repo_path.to_string_lossy()
        );

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert().success().stdout("");
        } else {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string))
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
        let mut cmd = create_cmd(&config_file, "deploy", &repo, quiet, verbose)?;
        if force {
            cmd.arg("--force");
        }
        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(repo_string));
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
    #[case(Some("rrcm-test".to_owned()), false, false)]
    #[case(None, false, false)]
    #[case(None, true, false)]
    #[case(None, false, true)]
    #[case(None, false, false)]
    fn test_deploy_error(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let temp = create_temp_dir()?;
        let repos = btreemap!(
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
        let repo_string = format!(
            r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {repo_path}
Cloning into '{repo_path}'...
"#,
            repo_path = repo_path.to_string_lossy()
        );

        for (path, _) in &deploy_files {
            OpenOptions::new().write(true).create(true).open(path)?;
        }

        let stderr_string = format!("{}{}", Red.paint("[ERROR] "), "Failed to deploy");

        // update clone
        let mut cmd = create_cmd(&config_file, "update", &repo, quiet, verbose)?;
        if quiet {
            cmd.assert()
                .success()
                .stdout("")
                .stderr(predicate::str::contains(&stderr_string));
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(&repo_string))
                .stderr(predicate::str::contains(&stderr_string));
        } else {
            cmd.assert()
                .success()
                .stdout(repo_string.clone())
                .stderr(predicate::str::contains(&stderr_string));
        }
        for (path, _) in &deploy_files {
            pretty_assertions::assert_eq!(path.is_symlink(), false);
        }

        let repo_string = "Repo rrcm-test\n";
        // status
        let mut cmd = create_cmd(&config_file, "status", &repo, quiet, verbose)?;
        cmd.assert()
            .success()
            .stdout(predicate::str::starts_with(repo_string))
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
