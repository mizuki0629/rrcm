//! Unix specific functions
//! This module is enabled only on Unix-like OS
//! (Linux, macOS, FreeBSD, etc.)
//! This module is disabled on Windows
//! because Windows has different path format
//! (e.g. C:\Users\username\...)
//! and different path separator (e.g. C:\Users\username\foo\bar.txt)
use anyhow::Context as _;
use anyhow::{bail, Ok, Result};
use dirs::{config_dir, data_dir, cache_dir, runtime_dir, state_dir};
use std::path::PathBuf;

/// Get default path of XDG Base Directory
/// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
/// | Variable | Default Value |
/// |----------|---------------|
/// | XDG_CONFIG_HOME | $HOME/.config |
/// | XDG_DATA_HOME | $HOME/.local/share |
/// | XDG_CACHE_HOME | $HOME/.cache |
/// | XDG_RUNTIME_DIR | /run/user/$UID |
/// | XDG_STATE_HOME | $HOME/.local/state |
fn get_xdg_default(s: &str) -> Result<String> {
    let mut path = PathBuf::new();
    match s {
        "XDG_CONFIG_HOME" => {
            path.push(config_dir().unwrap());
        }
        "XDG_DATA_HOME" => {
            path.push(data_dir().unwrap());
        }
        "XDG_CACHE_HOME" => {
            path.push(cache_dir().unwrap());
        }
        "XDG_RUNTIME_DIR" => {
            path.push(runtime_dir().unwrap());
        }
        "XDG_STATE_HOME" => {
            path.push(state_dir().unwrap());
        }
        _ => bail!("invalid XDG Base Directory: {}", s),
    }
    Ok(path.to_str().unwrap().to_string())
}

/// Check if the specified string is XDG Base Directory
/// XDG Base Directory is one of the following:
/// - XDG_CONFIG_HOME
/// - XDG_DATA_HOME
/// - XDG_CACHE_HOME
/// - XDG_RUNTIME_DIR
/// - XDG_STATE_HOME
fn is_xdg_base_directory(s: &str) -> bool {
    matches!(s, "XDG_CONFIG_HOME" | "XDG_DATA_HOME" | "XDG_CACHE_HOME" | "XDG_RUNTIME_DIR" | "XDG_STATE_HOME")
}

/// Expand environment variable in String
/// ${VARNAME} format is expanded
/// If VARNAME is XDG Base Directory, it is expanded to the path of the XDG Base Directory
/// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
pub fn expand_env_var(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars();

    #[allow(clippy::while_let_on_iterator)]
    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(c) = chars.next() {
                if c == '{' {
                    let mut varname = String::new();
                    let mut is_end = false;
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            is_end = true;
                            break;
                        }
                        varname.push(c);
                    }
                    if !is_end {
                        bail!("invalid env var name: {}", varname);
                    }
                    if varname.is_empty() {
                        bail!("invalid env var name: {}", varname);
                    }
                    if is_xdg_base_directory(&varname) {
                        result.push_str(&get_xdg_default(&varname)?);
                        continue;
                    }
                    let value = std::env::var(&varname)
                        .with_context(|| format!("env var {} not found", varname))?;
                    result.push_str(&value);
                } else {
                    result.push('$');
                    result.push(c);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_env_var() {
        let s = "${HOME}/.config";
        let result = expand_env_var(s).unwrap();
        assert_eq!(
            result,
            format!("{}/.config", std::env::var("HOME").unwrap())
        );

        let s = "${XDG_CONFIG_HOME}/.config";
        let result = expand_env_var(s).unwrap();
        assert_eq!(
            result,
            format!("{}/.config/.config", std::env::var("HOME").unwrap())
        );

        let s = "${XDG_CONFIG_HOME2}/.config";
        let result = expand_env_var(s);
        assert!(result.is_err());

        let s = "${XDG_CONFIG_HOME";
        let result = expand_env_var(s);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_xdg_base_directory() {
        assert!(is_xdg_base_directory("XDG_CONFIG_HOME"));
        assert!(is_xdg_base_directory("XDG_DATA_HOME"));
        assert!(is_xdg_base_directory("XDG_CACHE_HOME"));
        assert!(!is_xdg_base_directory("XDG_CONFIG_HOME2"));
    }

    #[test]
    fn test_get_xdg_default() {
        let result = get_xdg_default("XDG_CONFIG_HOME").unwrap();
        assert_eq!(
            result,
            format!("{}/.config", std::env::var("HOME").unwrap())
        );

        let result = get_xdg_default("XDG_DATA_HOME").unwrap();
        assert_eq!(
            result,
            format!("{}/.local/share", std::env::var("HOME").unwrap())
        );

        let result = get_xdg_default("XDG_CACHE_HOME").unwrap();
        assert_eq!(result, format!("{}/.cache", std::env::var("HOME").unwrap()));

        let result = get_xdg_default("XDG_CONFIG_HOME2");
        assert!(result.is_err());
    }
}
