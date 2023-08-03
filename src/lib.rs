mod config;
mod deploy_status;
mod fs;
mod path;
mod subcommand;

pub use subcommand::{deploy, print_error, status, undeploy, update};
