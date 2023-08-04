pub mod config;
mod deploy_status;
mod fs;
mod path;
mod subcommand;

pub use subcommand::{deploy, status, undeploy, update};
