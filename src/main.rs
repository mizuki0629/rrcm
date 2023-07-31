use anyhow::Result;
use clap::{Parser, Subcommand};
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
    #[clap(subcommand)]
    subcommand: SubCommands,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Initialize config file.
    Init {
        #[clap(required = true, ignore_case = true)]
        path: PathBuf,
    },
    /// Print deploy status.
    Status {
        #[clap(required = true, ignore_case = true)]
        path: PathBuf,
    },
    /// Deploy file or folder.
    Deploy {
        /// file or directory path
        #[clap(required = true, ignore_case = true)]
        path: Vec<PathBuf>,
        /// if eists file, remove and deploy.  
        #[clap(short, long, default_value_t = false)]
        force: bool,
    },
}

fn main() -> Result<()> {
    match Args::parse().subcommand {
        SubCommands::Init { path } => rrcm::init(path),
        SubCommands::Status { path } => rrcm::status(path),
        SubCommands::Deploy { path, force } => {
            for p in path {
                rrcm::deploy(&p, force)?
            }
            Ok(())
        }
    }
}
