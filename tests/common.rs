use maplit::btreemap;
use rrcm::config::AppConfig;
use rrcm::config::OsPath;
use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, WriteLogger};

fn init_logger(test_id: &str) {
    let tmpdir = env!("CARGO_TARGET_TMPDIR");
    let logdir = format!("{}/integration-tests/log", tmpdir);
    std::fs::create_dir_all(&logdir).unwrap_or_else(drop);
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            std::fs::File::create(format!("{}/{}.log", &logdir, test_id)).unwrap(),
        ),
        TermLogger::new(
            LevelFilter::Error,
            Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
    ])
    .unwrap_or_else(drop)
}

pub fn testdir(test_id: &str) -> String {
    let tmpdir = env!("CARGO_TARGET_TMPDIR");
    format!("{}/integration-tests/{}", tmpdir, test_id)
}

fn new_app_config(test_id: &str) -> AppConfig {
    let tmpdir = env!("CARGO_TARGET_TMPDIR");

    let dotfiles = OsPath {
        windows: Some(format!(
            "{}\\integration-tests\\{}\\dotfiles",
            tmpdir, test_id
        )),
        mac: Some(format!("{}/integration-tests/{}/dotfiles", tmpdir, test_id)),
        linux: Some(format!("{}/integration-tests/{}/dotfiles", tmpdir, test_id)),
    };

    let deploy = btreemap!(
        String::from("home") => OsPath {
            windows: Some(format!("{}\\integration-tests\\{}\\home",tmpdir, test_id)),
            mac: Some(format!("{}/integration-tests/{}/home",tmpdir, test_id)),
            linux: Some(format!("{}/integration-tests/{}/home",tmpdir, test_id)),
        },
        String::from("config") => OsPath {
            windows: Some(format!("{}\\integration-tests\\{}\\config",tmpdir, test_id)),
            mac: Some(format!("{}/integration-tests/{}/config",tmpdir, test_id)),
            linux: Some(format!("{}/integration-tests/{}/config",tmpdir, test_id)),
        },
        String::from("config_local") => OsPath {
            windows: Some(format!("{}\\integration-tests\\{}\\config_local",tmpdir, test_id)),
            mac: Some(format!("{}/integration-tests/{}/config_local",tmpdir, test_id)),
            linux: Some(format!("{}/integration-tests/{}/config_local",tmpdir, test_id)),
        },
    );

    let repos = btreemap!(
            String::from("rrcm-test") => String::from("https://github.com/mizuki0629/rrcm-test.git"),
    );

    AppConfig {
        dotfiles,
        deploy,
        repos,
    }
}

fn setup_directory(test_id: &str) {
    let testdir = testdir(test_id);
    let dotfiles = format!("{}/dotfiles", testdir);
    let home = format!("{}/home", testdir);
    let config = format!("{}/config", testdir);
    let config_local = format!("{}/config_local", testdir);

    std::fs::create_dir_all(dotfiles).unwrap();
    std::fs::create_dir_all(home).unwrap();
    std::fs::create_dir_all(config).unwrap();
    std::fs::create_dir_all(config_local).unwrap();
}

fn teardown_directory(test_id: &str) {
    let testdir = testdir(test_id);

    std::fs::remove_dir_all(testdir).unwrap();
}

pub fn setup(test_id: &str) -> AppConfig {
    init_logger(test_id);

    setup_directory(test_id);

    new_app_config(test_id)
}

pub fn teardown(test_id: &str) {
    teardown_directory(test_id);
}
