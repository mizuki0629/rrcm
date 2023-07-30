#[cfg(not(target_os = "windows"))]
mod unix;

#[cfg(not(target_os = "windows"))]
pub use crate::path::unix::expand_env_var;


#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use crate::path::windows::expand_env_var;
