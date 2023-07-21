mod fs;
mod subcommand;
mod appconfig;

use anyhow;
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
    /// Print deploy status.
    Status {
        /// Print all status
        #[clap(short, long, default_value_t = false)]
        all: bool,
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

fn main() -> anyhow::Result<()> {
    match Args::parse().subcommand {
        SubCommands::Status { all } => {
            subcommand::status(all)
        }
        SubCommands::Deploy { path, force } => {
            for p in path {
                subcommand::deploy(&p, force)?
            }
            Ok(())
        }
    }
}
