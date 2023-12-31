//! ## Introduction
//! - Deploy configuration files and directories using symbolic links.
//! - Configuration files and directories are managed in a git repository.
//! - Provides deployment on multiple OS from the same directory.
//!
//! Provides the location of these directories by leveraging the mechanisms defined by
//! - the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)  specifications on Linux and macOS
//! - the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows
//!
//! ## Direcotry
//! The configuration file specifies the mapping between deployment sources and destinations for each platform.
//! rrcm creates symbolic links to files and directories directly under the deployment targets in the deployment destination.
//! ```text
//! (dotfiles repository download directory)
//! dotiles
//! │   (local repositroy)
//! ├── example1
//! │   │   (deploy target)
//! │   ├── home
//! │   │   │               (symbolic link)
//! │   │   ├── .profile <- $HOME/.profile     (on Linux and Mac)
//! │   │   │               %PROFILE%\.profile (on Win)
//! │   │   ├── .vim     <- $HOME/.vim         (on Linux and Mac)
//! │   │   │               %PROFILE%\.vim     (on Win)
//! │   │   └── ...
//! │   │   
//! │   └── .config
//! │       ├── nvim     <- $HOME/.config/nvim                  (on Linux and Mac),
//! │       │               %PROFILE%\AppData\LocalAppData\nvim (on Win)
//! │       └── ...
//! │   
//! └── example2 (next local repositroy)
//!     └── ...
//! ```
//! Under the deployment target dierctory, files and directorys are deployed by symbolic link.
//! **Windows needs to be run as administrator.**
//!
//!
//! ## Configuration
//! configuration file path:
//! - Unix: $HOME/.config/rrcm/config.yaml
//! - Win: %PROFILE%\AppData\Roaming\rrcm\config.yaml
//! ```yaml
//! ---
//! # dotfiles repositroy download directory
//! dotfiles:
//!   windows: "%USERPROFILE%\\dotfiles"
//!   mac: "${HOME}/.dotfiles"
//!   linux: "${HOME}/.dotfiles"
//!
//! # repositories. multiple repositories can be specified.
//! repos:
//!
//!     # local repository name
//!   - name: example1
//!
//!     # git repository url
//!     url: 'git@github:example/example1'
//!
//!     # deploy configuration
//!     deploy:
//!
//!       # deploy target
//!       # Example: deploy home directory to $HOME or %USERPROFILE%
//!       home:
//!
//!         # deploy destination on each OS.
//!         # if OS not defined, it will not be deployed.
//!         windows: "%USERPROFILE%"
//!         mac: "${HOME}"
//!         linux: "${HOME}"
//!
//!       # Example: deploy .config directory to XDG_CONFIG_HOME or %USERPROFILE%\AppData\Roaming
//!       .config:
//!         windows: "%FOLDERID_RoamingAppData%"
//!         mac: "${XDG_CONFIG_HOME}"
//!         linux: "${XDG_CONFIG_HOME}"
//!
//!       # Example: deploy .config-LocalAppData to XDG_CONFIG_HOME or %USERPROFILE%\AppData\Local
//!       .config-LocalAppData:
//!         windows: "%FOLDERID_LocalAppData%"
//!         mac: "${XDG_CONFIG_HOME}"
//!         linux: "${XDG_CONFIG_HOME}"
//!
//!     # next repository
//!   - name: example2
//!     url: 'git@github:example/example2'
//!     ...
//! ```
//!
//! ### Deployment destination
//! Environment variables can be used in deployment destination.
//!
//! Format
//! - Unix: ${ENVIRONMENT_VARIABLE_NAME}
//! - Windows: %ENVIRONMENT_VARIABLE_NAME%
//!
//! The following special variables are available.
//! - Unix [XDG base directory](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
//!     if the environment variable is not set, the default value is used.
//!     - ${XDG_CONFIG_HOME}
//!     - ${XDG_DATA_HOME}
//! - Windows [Known Folder ID](https://docs.microsoft.com/en-us/windows/win32/shell/knownfolderid)
//!     - %FOLDERID_RoamingAppData%
//!     - %FOLDERID_LocalAppData%
//!     - %FOLDERID_Documents%
//!     - %FOLDERID_Desktop%
//!
//! ## Install
//! ### Cargo
//! ```sh
//! cargo install rrcm
//! ```
//!
//! ## Usage
//! initialize config file
//! ```sh
//! rrcm init
//! # or initialize config file from http
//! rrcm init <url>
//! ```
//! I recommend using gist like this.
//! [my config.yaml](https://gist.github.com/mizuki0629/1f7e73703b09551610b18392e375bd73)
//!
//! update(git clone or pull) repositories and deploy
//! ```sh
//! rrcm update
//! ```
//!
//! show deploy status
//! ```sh
//! rrcm status
//! ```
//!
use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};
use std::path::PathBuf;
use url::Url;

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
    Init {
        /// rrcm file url
        #[clap(required = false, ignore_case = true)]
        url: Option<Url>,
    },
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
        /// if eists file, remove and deploy.  
        #[clap(short, long, default_value_t = false)]
        force: bool,
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
            SubCommands::Init { url } => {
                // download config file if url is specified
                if let Some(url) = url {
                    rrcm::config::download_app_config(&config, url)?;
                } else {
                    rrcm::config::init_app_config(&config)?;
                }
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
            SubCommands::Update { ref repo, force } => {
                let app_config = rrcm::config::load_app_config(&config)?;
                rrcm::update(
                    &app_config,
                    repo,
                    args.log.quiet,
                    args.log.verbose || args.log.debug || args.log.trace,
                    force,
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
