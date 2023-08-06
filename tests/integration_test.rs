use std::fs;

use anyhow::Result;
mod common;
use common::{setup, teardown};
use std::path::Path;
use std::path::PathBuf;

mod win_need_admin {
    use super::*;

    #[test]
    fn test_update() -> Result<()> {
        let test_id = "update";
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, false, false)?;

        let testdir = common::testdir(test_id);
        assert_eq!(
            fs::read_link(format!("{}/home/.test.cfg", testdir))?,
            PathBuf::from(format!("{}/dotfiles/rrcm-test/home/.test.cfg", testdir))
        );

        teardown(test_id);
        Ok(())
    }

    #[test]
    fn test_undeploy() -> Result<()> {
        let test_id = "undeploy";
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, false, false)?;
        rrcm::undeploy(&app_config, None, false)?;

        let testdir = common::testdir(test_id);
        assert!(!Path::new(&format!("{}/home/.test.cfg", testdir)).exists());

        teardown(test_id);
        Ok(())
    }

    #[test]
    fn test_deploy() -> Result<()> {
        let test_id = "deploy";
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, false, false)?;
        rrcm::undeploy(&app_config, None, false)?;
        rrcm::deploy(&app_config, None, false, false)?;

        let testdir = common::testdir(test_id);
        assert!(Path::new(&format!("{}/home/.test.cfg", testdir)).exists());

        teardown(test_id);
        Ok(())
    }

    #[test]
    fn test_status_deployed() -> Result<()> {
        let test_id = "status_deployed";
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, false, false)?;
        rrcm::status(&app_config, None)?;

        teardown(test_id);
        Ok(())
    }

    #[test]
    fn test_status_undeployed() -> Result<()> {
        let test_id = "status_undeployed";
        let app_config = setup(test_id);

        rrcm::update(&app_config, None, false, false)?;
        rrcm::undeploy(&app_config, None, false)?;
        rrcm::status(&app_config, None)?;

        teardown(test_id);
        Ok(())
    }
}

#[test]
fn test_status_empty() -> Result<()> {
    let test_id = "status_empty";
    let app_config = setup(test_id);

    rrcm::status(&app_config, None)?;

    teardown(test_id);
    Ok(())
}
