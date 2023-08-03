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
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
struct Args {
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Print deploy status.
    Status {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,

        /// Be verbose
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
    },
    /// Deploy file or folder.
    Deploy {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,
        /// Be quiet
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
        /// if eists file, remove and deploy.  
        #[clap(short, long, default_value_t = false)]
        force: bool,
    },
    /// Deploy file or folder.
    Undeploy {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,

        /// Be quiet
        #[clap(short, long, default_value_t = false)]
        quiet: bool,
    },
    /// Update repository.
    Update {
        /// repository name
        #[clap(required = false, ignore_case = true)]
        repo: Option<String>,

        /// Be quiet
        #[clap(short, long, default_value_t = false)]
        quiet: bool,

        /// Be verbose
        #[clap(short, long, default_value_t = false)]
        verbose: bool,
    },
}

fn main() {
    match Args::parse().subcommand {
        SubCommands::Status { repo, verbose } => rrcm::status(repo, verbose),
        SubCommands::Deploy { repo, quiet, force } => rrcm::deploy(repo, quiet, force),
        SubCommands::Undeploy { repo, quiet } => rrcm::undeploy(repo, quiet),
        SubCommands::Update {
            repo,
            quiet,
            verbose,
        } => rrcm::update(repo, quiet, verbose),
    }
    .unwrap_or_else(|e| {
        rrcm::print_error(&e);
        std::process::exit(1);
    });
}
