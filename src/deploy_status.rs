use crate::fs;
use core::fmt::{self, Display};
use core::hash::Hash;
use std::fs::read_link;
use std::path::Path;

#[derive(Debug, Eq, Clone)]
pub enum DeployStatus {
    UnDeployed,
    Deployed,
    Conflict { cause: String },
    UnManaged,
}
impl PartialEq for DeployStatus {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (DeployStatus::UnDeployed, DeployStatus::UnDeployed)
                | (DeployStatus::Deployed, DeployStatus::Deployed)
                | (DeployStatus::UnManaged, DeployStatus::UnManaged)
                | (DeployStatus::Conflict { .. }, DeployStatus::Conflict { .. })
        )
    }
}

impl Hash for DeployStatus {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DeployStatus::UnDeployed => 0.hash(state),
            DeployStatus::Deployed => 1.hash(state),
            DeployStatus::UnManaged => 2.hash(state),
            DeployStatus::Conflict { .. } => 3.hash(state),
        }
    }
}

impl Display for DeployStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeployStatus::UnDeployed => write!(f, "UnDeployed"),
            DeployStatus::Deployed => write!(f, "Deployed"),
            DeployStatus::UnManaged => write!(f, "UnManaged"),
            DeployStatus::Conflict { .. } => write!(f, "Conflict"),
        }
    }
}

pub fn get_status<P, Q>(from: P, to: Q) -> DeployStatus
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let Some(from) = from.as_ref().exists().then_some(from) else {
        return DeployStatus::UnManaged;
    };

    if !to.as_ref().is_symlink() {
        if to.as_ref().exists() {
            return DeployStatus::Conflict {
                cause: format!("Other file exists. {}", to.as_ref().to_string_lossy()),
            };
        } else {
            return DeployStatus::UnDeployed;
        }
    }

    let abs_to_link = fs::absolutize(read_link(to).unwrap()).unwrap();
    if fs::absolutize(from).unwrap() != abs_to_link {
        return DeployStatus::Conflict {
            cause: format!(
                "Symlink to different path. {}",
                abs_to_link.to_string_lossy()
            ),
        };
    }

    DeployStatus::Deployed
}