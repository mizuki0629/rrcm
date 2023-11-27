//! Unix specific functions
//! This module is enabled only on Unix-like OS
//! (Linux, macOS, FreeBSD, etc.)
//! This module is disabled on Windows
//! because Windows has different path format
//! (e.g. C:\Users\username\...)
//! and different path separator (e.g. C:\Users\username\foo\bar.txt)
use anyhow::Context as _;
use anyhow::{bail, Ok, Result};
use dirs::{cache_dir, config_dir, data_dir, state_dir};
use std::path::PathBuf;

/// Get default path of XDG Base Directory
/// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
///
/// | Variable | Default Value |
/// |----------|---------------|
/// | XDG_CONFIG_HOME | $HOME/.config |
/// | XDG_DATA_HOME | $HOME/.local/share |
/// | XDG_CACHE_HOME | $HOME/.cache |
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
/// - XDG_STATE_HOME
fn is_xdg_base_directory(s: &str) -> bool {
    matches!(
        s,
        "XDG_CONFIG_HOME" | "XDG_DATA_HOME" | "XDG_CACHE_HOME" | "XDG_STATE_HOME"
    )
}

/// Expand environment variable in String
/// ${VARNAME} format is expanded
/// If VARNAME is XDG Base Directory, it is expanded to the path of the XDG Base Directory
/// https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
pub fn expand_env_var(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(c) = chars.next() {
                if c == '{' {
                    let mut varname = String::new();
                    let mut is_end = false;
                    for c in chars.by_ref() {
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
                        bail!("variable name is empty");
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
    use rstest::rstest;

    #[rstest]
    #[case("${HOME}", format!("{}", std::env::var("HOME").unwrap()))]
    #[case("${HOME}/.config", format!("{}/.config", std::env::var("HOME").unwrap()))]
    #[case("${HOME}/.config/${XDG_CONFIG_HOME}", format!("{}/.config/{}/.config", std::env::var("HOME").unwrap(), std::env::var("HOME").unwrap()))]
    #[case("${HOME}/.config/${XDG_CONFIG_HOME}/foo", format!("{}/.config/{}/.config/foo", std::env::var("HOME").unwrap(), std::env::var("HOME").unwrap()))]
    fn test_expand_env_var(#[case] s: &str, #[case] expected: String) -> Result<()> {
        let result = expand_env_var(s)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[rstest]
    #[case("${HOME", "invalid env var name: HOME")]
    #[case("${}/.config", "variable name is empty")]
    #[case("${HOME2}/.config", "env var HOME2 not found")]
    #[case(
        "${HOME}/.config/${XDG_CONFIG_HOME",
        "invalid env var name: XDG_CONFIG_HOME"
    )]
    #[case(
        "${HOME}/.config/${XDG_CONFIG_HOME2}/foo",
        "env var XDG_CONFIG_HOME2 not found"
    )]
    fn test_expand_env_var_error(#[case] s: &str, #[case] expected: &str) {
        let result = expand_env_var(s);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), expected);
    }

    #[rstest]
    #[case("XDG_CONFIG_HOME", true)]
    #[case("XDG_DATA_HOME", true)]
    #[case("XDG_CACHE_HOME", true)]
    #[case("XDG_STATE_HOME", true)]
    #[case("XDG_CONFIG_HOME2", false)]
    fn test_is_xdg_base_directory(#[case] s: &str, #[case] expected: bool) {
        assert_eq!(is_xdg_base_directory(s), expected);
    }

    #[rstest]
    #[case("XDG_CONFIG_HOME", format!("{}/.config", std::env::var("HOME").unwrap()))]
    #[case("XDG_DATA_HOME", format!("{}/.local/share", std::env::var("HOME").unwrap()))]
    #[case("XDG_CACHE_HOME", format!("{}/.cache", std::env::var("HOME").unwrap()))]
    #[case("XDG_STATE_HOME", format!("{}/.local/state", std::env::var("HOME").unwrap()))]
    fn test_get_xdg_default(#[case] s: &str, #[case] expected: String) -> Result<()> {
        let result = get_xdg_default(s)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[rstest]
    #[case("XDG_CONFIG_HOME2")]
    fn test_get_xdg_default_error(#[case] s: &str) -> Result<()> {
        let result = get_xdg_default(s);
        assert!(result.is_err());
        Ok(())
    }
}
