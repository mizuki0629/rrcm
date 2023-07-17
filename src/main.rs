mod fs;
mod subcommand;

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
        #[clap(default_value = ".")]
        path: PathBuf,
        /// if eists file, remove and deploy.  
        #[clap(short, long, default_value_t = false)]
        simple: bool,
    },
    /// Add managed file or folder.
    Add {
        /// file or directory path
        #[clap(required = true, ignore_case = true)]
        path: Vec<PathBuf>,
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
    /// Print
    List {
        /// file or directory path
        #[clap(required = true, ignore_case = true)]
        path: Vec<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    match Args::parse().subcommand {
        SubCommands::Status { path, simple } => {
            subcommand::status(path.as_path(), simple)
        }
        SubCommands::List { path } => {
            subcommand::list(path)
        },
        SubCommands::Add { path } => {
            for p in path {
                subcommand::add(p.as_path())?;
            }
            Ok(())
        }
        SubCommands::Deploy { path, force } => {
            for p in path {
                subcommand::deploy(p.as_path(), force)?
            }
            Ok(())
        }
    }
}
