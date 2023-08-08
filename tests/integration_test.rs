use std::fs;

use anyhow::Result;
mod common;
use common::{setup, teardown};
use std::path::Path;
use std::path::PathBuf;

#[test]
fn test_load_app_config() -> Result<()> {
    let test_id = "load_app_config";
    let app_config = setup(test_id);
    let path = PathBuf::from(common::testdir(test_id)).join("config.toml");
    confy::store_path(&path, app_config)?;

    rrcm::config::load_app_config(&path)?;

    teardown(test_id);
    Ok(())
}

#[test]
fn test_load_app_config_not_exists() -> Result<()> {
    let test_id = "load_app_config_not_exists";
    let _ = setup(test_id);
    let path = PathBuf::from(common::testdir(test_id)).join("config.toml");

    assert!(rrcm::config::load_app_config(path).is_err());

    teardown(test_id);
    Ok(())
}

#[test]
fn test_load_app_config_invalid() -> Result<()> {
    let test_id = "load_app_config_invalid";
    let _ = setup(test_id);
    let path = PathBuf::from(common::testdir(test_id)).join("config.toml");
    fs::write(&path, "invalid")?;

    assert!(rrcm::config::load_app_config(&path).is_err());

    teardown(test_id);
    Ok(())
}

#[test]
fn test_status_empty() -> Result<()> {
    let test_id = "status_empty";
    let app_config = setup(test_id);

    rrcm::status(&app_config, None)?;

    teardown(test_id);
    Ok(())
}

mod win_need_admin {

    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("update_case_1", false, false)]
    #[case("update_case_2", true, false)]
    #[case("update_case_3", false, true)]
    fn test_update(
        #[case] test_id: &str,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, quiet, verbose)?;
        rrcm::status(&app_config, None)?;

        let testdir = common::testdir(test_id);
        assert_eq!(
            fs::read_link(format!("{}/home/.test.cfg", testdir))?,
            PathBuf::from(format!("{}/dotfiles/rrcm-test/home/.test.cfg", testdir))
        );

        assert_eq!(
            fs::read_link(format!("{}/config_local/dir", testdir))?,
            PathBuf::from(format!("{}/dotfiles/rrcm-test/config_local/dir", testdir))
        );

        teardown(test_id);
        Ok(())
    }

    #[rstest]
    #[case("undeploy_case_1", false, false)]
    #[case("undeploy_case_2", true, false)]
    #[case("undeploy_case_3", false, true)]
    fn test_undeploy(
        #[case] test_id: &str,
        #[case] quiet: bool,
        #[case] verbose: bool,
    ) -> Result<()> {
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, quiet, verbose)?;
        rrcm::undeploy(&app_config, None, quiet)?;
        rrcm::status(&app_config, None)?;

        let testdir = common::testdir(test_id);
        assert!(!Path::new(&format!("{}/home/.test.cfg", testdir)).exists());
        assert!(!Path::new(&format!("{}/config_local/dir", testdir)).exists());

        teardown(test_id);
        Ok(())
    }

    #[rstest]
    #[case("deploy_case_1", false, false, false)]
    #[case("deploy_case_2", true, false, false)]
    #[case("deploy_case_3", false, true, false)]
    #[case("deploy_case_4", false, false, true)]
    fn test_deploy(
        #[case] test_id: &str,
        #[case] quiet: bool,
        #[case] verbose: bool,
        #[case] force: bool,
    ) -> Result<()> {
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, quiet, verbose)?;
        rrcm::undeploy(&app_config, None, quiet)?;
        rrcm::deploy(&app_config, None, quiet, force)?;
        rrcm::status(&app_config, None)?;

        let testdir = common::testdir(test_id);
        assert!(Path::new(&format!("{}/home/.test.cfg", testdir)).exists());
        assert!(Path::new(&format!("{}/config_local/dir", testdir)).exists());

        teardown(test_id);
        Ok(())
    }
}
