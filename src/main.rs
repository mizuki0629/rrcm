//! ## Introduction
//! - Deploy configuration files and directories using symbolic links.
//! - Configuration files and directories are managed in a git repository.
//! - Provides deployment on multiple OS from the same directory.
//!
//! Provides the location of these directories by leveraging the mechanisms defined by
//! - the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)  specifications on Linux and macOS
//! - the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows
//!
//! ## Installation
//! ### Cargo
//! ```sh
//! cargo install rrcm
//! ```
//!
//! ## Configuration
//! The configuration file is a TOML file.
//! configuration file path:
//! - Unix: $HOME/.config/rrcm/config.toml
//! - Win: %PROFILE%\AppData\Roaming\rrcm\config.toml
//!
//! ### Repository
//! The repository is defined in the config.toml file.
//! ```toml
//! [repos]
//! # your dotfiles repository
//! # you can define multiple repositories
//! example1 = "git@github:example/example1"
//! example2 = "git@github:example/example2"
//! ```
//! the repository is a directory that contains the dotfiles.
//! Directory structure example:
//! ```
//! dotfiles
//! ├── home
//! │   ├── .profile -> $HOME/.profile(Unix)
//! │   │              %PROFILE%\.profile(Win)
//! │   └── ...
//! ├── config
//! │   ├── nushell  -> $HOME/.config/nushell(Unix),
//! │   │               %PROFILE%\AppData\Roaming\nushell(Win)
//! │   └── ...
//! └── config_local
//!     ├── nvim     -> $HOME/.config/nvim(Unix),
//!     │               %PROFILE%\AppData\Local\nvim(Win)
//!     └── ...
//! ```
//! home, config, config_local are the deployment targets.
//! - home: Deploy to $HOME(Unix), %PROFILE%(Win)
//! - config: Deploy to $HOME/.config(Unix), %PROFILE%\AppData\Roaming(Win)
//! - config_local: Deploy to $HOME/.config/local(Unix), %PROFILE%\AppData\Local(Win)
//!
//! Under the deployment target, the file or directory is deployed by symbolic link.
//! **Windows needs to be run as administrator.**
//!
//! ### Deployment target
//! The deployment target is defined in the config.toml file.
//! ```toml
//! [deploy.config]
//! windows = '%FOLDERID_RoamingAppData%'
//! mac = '${XDG_CONFIG_HOME}'
//! linux = '${XDG_CONFIG_HOME}'
//!
//! [deploy.config_local]
//! windows = '%FOLDERID_LocalAppData%'
//! mac = '${XDG_CONFIG_HOME}'
//! linux = '${XDG_CONFIG_HOME}'
//!
//! [deploy.home]
//! windows = '%USERPROFILE%'
//! mac = '${HOME}'
//! linux = '${HOME}'
//! ```
//!
//! Environment variables can be used in the deployment target.
//! Format
//! - Unix: ${ENVIRONMENT_VARIABLE_NAME}
//! - Windows: %ENVIRONMENT_VARIABLE_NAME%
//!
//! The following special variables are available.
//! - Unix [XDG base directory](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
//!     if the environment variable is not set, the default value is used.
//!     - ${XDG_CONFIG_HOME}
//!     - ${XDG_DATA_HOME}
//!     - ${XDG_CACHE_HOME}
//!     - ${XDG_RUNTIME_DIR}
//! - Windows [Known Folder ID](https://docs.microsoft.com/en-us/windows/win32/shell/knownfolderid)
//!     - %FOLDERID_RoamingAppData%
//!     - %FOLDERID_LocalAppData%
//!     - %FOLDERID_Documents%
//!     - %FOLDERID_Desktop%
//!
//! ## Examples
//! ```sh
//! # update all repositories
//! rrcm update
//!
//! # show status all repositories
//! rrcm status
//! ```
//!
use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
struct Args {
    #[command(flatten)]
    log: LogArgs,

    /// config file path
    #[clap(required = false, short, long)]
    config: Option<PathBuf>,

    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(clap::Args, Debug)]
#[group(required = false, multiple = false)]
struct LogArgs {
    /// Be verbose
    #[clap(short, long, default_value_t = false)]
    verbose: bool,

    /// Print debug information
    #[clap(short, long, default_value_t = false)]
    debug: bool,

    /// Print trace information
    #[clap(short, long, default_value_t = false)]
    trace: bool,

    /// Be quiet
    #[clap(short, long, default_value_t = false)]
    quiet: bool,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Initialize configuration file.
    Init {},
    /// Print deploy status.
    Status {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,
    },
    /// Deploy file or folder.
    Deploy {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,
        /// if eists file, remove and deploy.  
        #[clap(short, long, default_value_t = false)]
        force: bool,
    },
    /// Deploy file or folder.
    Undeploy {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,
    },
    /// Update repository.
    Update {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,
    },
}

fn main() {
    (|| {
        let args = Args::parse();
        init_logger(&args.log)?;

        let config = if let Some(ref path) = args.config {
            path.clone()
        } else {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("config directory not found"))?
                .join("rrcm")
                .join("config.yaml")
        };

        match args.subcommand {
            SubCommands::Init {} => {
                rrcm::config::init_app_config(&config)?;
            }
            SubCommands::Status { ref repo } => {
                let app_config = rrcm::config::load_app_config(&config)?;
                rrcm::status(&app_config, repo)?;
            }
            SubCommands::Deploy { ref repo, force } => {
                let app_config = rrcm::config::load_app_config(&config)?;
                rrcm::deploy(&app_config, repo, args.log.quiet, force)?;
            }
            SubCommands::Undeploy { ref repo } => {
                let app_config = rrcm::config::load_app_config(&config)?;
                rrcm::undeploy(&app_config, repo, args.log.quiet)?;
            }
            SubCommands::Update { ref repo } => {
                let app_config = rrcm::config::load_app_config(&config)?;
                rrcm::update(
                    &app_config,
                    repo,
                    args.log.quiet,
                    args.log.verbose || args.log.debug || args.log.trace,
                )?;
            }
        }
        Ok(())
    })()
    .unwrap_or_else(|e| {
        log::error!("{:?}", e);
        std::process::exit(1);
    });
}

fn get_log_level(log: &LogArgs) -> LevelFilter {
    if log.quiet {
        LevelFilter::Error
    } else if log.trace {
        LevelFilter::Trace
    } else if log.debug {
        LevelFilter::Debug
    } else if log.verbose {
        LevelFilter::Info
    } else {
        LevelFilter::Error
    }
}

fn init_logger(log: &LogArgs) -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        get_log_level(log),
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(false, false, false, false, LevelFilter::Error)]
    #[case(false, false, false, true, LevelFilter::Info)]
    #[case(false, false, true, false, LevelFilter::Debug)]
    #[case(false, true, false, false, LevelFilter::Trace)]
    #[case(true, false, false, false, LevelFilter::Error)]
    fn test_get_log_level(
        #[case] quiet: bool,
        #[case] trace: bool,
        #[case] debug: bool,
        #[case] verbose: bool,
        #[case] expected: LevelFilter,
    ) {
        let log = LogArgs {
            quiet,
            trace,
            debug,
            verbose,
        };
        assert_eq!(get_log_level(&log), expected);
    }
}
