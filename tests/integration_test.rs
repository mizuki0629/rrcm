use ansi_term::Colour::{Green, Yellow};
use anyhow::Result;
use assert_cmd::Command;
use assert_fs::fixture::ChildPath;
use assert_fs::prelude::*;
use maplit::btreemap;
use predicates::prelude::*;
use rrcm::config::AppConfig;
use rrcm::config::OsPath;
use std::fs;
use std::path::MAIN_SEPARATOR;

fn create_app_config(temp: &assert_fs::TempDir) -> Result<ChildPath> {
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

    let repos = btreemap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
    );

    config_file.write_str(&toml::to_string(&AppConfig {
        dotfiles,
        deploy,
        repos,
    })?)?;

    Ok(config_file)
}

mod win_need_admin {

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false)]
    #[case(None, false, false)]
    #[case(None, true, false)]
    #[case(None, false, true)]
    fn test_update_clone(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let config_file = create_app_config(&temp)?;

        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("update");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }

        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(format!(
                    r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                )))
                .stdout(predicate::str::ends_with(format!(
                    r#"{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                    status = Green.paint("    Deployed"),
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                temp = temp.path().to_string_lossy(),
                delim = MAIN_SEPARATOR,
                status = Green.paint("    Deployed"),
            ));
        }

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );

        temp.close()?;
        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false)]
    #[case(None, false, false)]
    #[case(None, true, false)]
    #[case(None, false, true)]
    fn test_update_pull(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let config_file = create_app_config(&temp)?;

        // update clone
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("update");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }

        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(format!(
                    r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                )))
                .stdout(predicate::str::ends_with(format!(
                    r#"{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                    status = Green.paint("    Deployed"),
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                temp = temp.path().to_string_lossy(),
                delim = MAIN_SEPARATOR,
                status = Green.paint("    Deployed"),
            ));
        }

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );

        // update pull
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("update");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }
        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(format!(
                    r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
"#,
                temp = temp.path().to_string_lossy(),
                delim = MAIN_SEPARATOR,
            ));
        }

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );

        temp.close()?;
        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false)]
    #[case(None, false, false)]
    #[case(None, true, false)]
    #[case(None, false, true)]
    fn test_undeploy(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let temp = assert_fs::TempDir::new()?;
        let config_file = create_app_config(&temp)?;

        // update clone
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("update");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }

        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(format!(
                    r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                )))
                .stdout(predicate::str::ends_with(format!(
                    r#"{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                    status = Green.paint("    Deployed"),
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                temp = temp.path().to_string_lossy(),
                delim = MAIN_SEPARATOR,
                status = Green.paint("    Deployed"),
            ));
        }

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );

        // undeploy
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("undeploy");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }
        let assert = cmd.assert();
        assert.success();

        pretty_assertions::assert_eq!(false, temp.path().join("home").join(".test.cfg").exists());

        pretty_assertions::assert_eq!(false, temp.path().join("config_local").join("dir").exists());

        // status
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("status");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }

        if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(
                    r#"Repo rrcm-test
"#,
                ))
                .stdout(predicate::str::ends_with(format!(
                    r#"{status} config_local{delim}dir
{status} home{delim}.test.cfg
"#,
                    status = Yellow.paint("  UnDeployed"),
                    delim = MAIN_SEPARATOR,
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Repo rrcm-test
{status} config_local{delim}dir
{status} home{delim}.test.cfg
"#,
                status = Yellow.paint("  UnDeployed"),
                delim = MAIN_SEPARATOR,
            ));
        }

        temp.close()?;
        Ok(())
    }

    #[rstest]
    #[case(Some("rrcm-test".to_owned()), false, false, false)]
    #[case(None, false, false, false)]
    #[case(None, true, false, false)]
    #[case(None, false, true, false)]
    #[case(None, false, false, true)]
    fn test_deploy(
        #[case] repo: Option<String>,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] force: bool,
    ) -> Result<()> {
        let temp = assert_fs::TempDir::new()?.into_persistent();
        let config_file = create_app_config(&temp)?;

        // update
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("update");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }

        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(format!(
                    r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                )))
                .stdout(predicate::str::ends_with(format!(
                    r#"{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                    status = Green.paint("    Deployed"),
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                temp = temp.path().to_string_lossy(),
                delim = MAIN_SEPARATOR,
                status = Green.paint("    Deployed"),
            ));
        }

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );

        // undeploy
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("undeploy");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }
        let assert = cmd.assert();
        assert.success();

        pretty_assertions::assert_eq!(false, temp.path().join("home").join(".test.cfg").exists());

        pretty_assertions::assert_eq!(false, temp.path().join("config_local").join("dir").exists());

        // deploy
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("deploy");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }
        if force {
            cmd.arg("--force");
        }
        let assert = cmd.assert();
        assert.success();

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );

        // status
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("status");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }

        if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(
                    r#"Repo rrcm-test
"#,
                ))
                .stdout(predicate::str::ends_with(format!(
                    r#"{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                    status = Green.paint("    Deployed"),
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Repo rrcm-test
{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                temp = temp.path().to_string_lossy(),
                delim = MAIN_SEPARATOR,
                status = Green.paint("    Deployed"),
            ));
        }

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
        let temp = assert_fs::TempDir::new()?;
        let config_file = create_app_config(&temp)?;

        // update
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("update");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }

        if quiet {
            cmd.assert().success().stdout("");
        } else if verbose {
            cmd.assert()
                .success()
                .stdout(predicate::str::starts_with(format!(
                    r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                )))
                .stdout(predicate::str::ends_with(format!(
                    r#"{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                    temp = temp.path().to_string_lossy(),
                    delim = MAIN_SEPARATOR,
                    status = Green.paint("    Deployed"),
                )));
        } else {
            cmd.assert().success().stdout(format!(
                r#"Update rrcm-test
  https://github.com/mizuki0629/rrcm-test.git => {temp}{delim}dotfiles{delim}rrcm-test
Cloning into '{temp}{delim}dotfiles{delim}rrcm-test'...
{status} {temp}{delim}config_local{delim}dir
{status} {temp}{delim}home{delim}.test.cfg
"#,
                temp = temp.path().to_string_lossy(),
                delim = MAIN_SEPARATOR,
                status = Green.paint("    Deployed"),
            ));
        }

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );

        // deploy
        let mut cmd = Command::cargo_bin("rrcm")?;
        if quiet {
            cmd.arg("--quiet");
        }
        if verbose {
            cmd.arg("--verbose");
        }
        cmd.arg("--config").arg(config_file.path());
        cmd.arg("deploy");
        if let Some(repo) = &repo {
            cmd.arg(repo);
        }
        if force {
            cmd.arg("--force");
        }
        let assert = cmd.assert();
        assert.success();

        pretty_assertions::assert_eq!(true, temp.path().join("home").join(".test.cfg").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("home").join(".test.cfg"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("home")
                .join(".test.cfg")
        );

        pretty_assertions::assert_eq!(true, temp.path().join("config_local").join("dir").exists());
        pretty_assertions::assert_eq!(
            fs::read_link(temp.path().join("config_local").join("dir"))?,
            temp.path()
                .join("dotfiles")
                .join("rrcm-test")
                .join("config_local")
                .join("dir")
        );
        temp.close()?;

        Ok(())
    }
}
