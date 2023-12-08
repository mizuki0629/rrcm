# rrcm
[![Crates.io](https://img.shields.io/crates/v/rrcm.svg)](https://crates.io/crates/rrcm)
[![Workflow Status](https://github.com/mizuki0629/rrcm/workflows/Test/badge.svg)](https://github.com/mizuki0629/rrcm/actions?query=workflow%3A%22Test%22)
[![codecov](https://codecov.io/gh/mizuki0629/rrcm/branch/master/graph/badge.svg?token=IVPHQ5UQIL)](https://codecov.io/gh/mizuki0629/rrcm)

## Introduction
- Deploy configuration files and directories using symbolic links.
- Configuration files and directories are managed in a git repository.
- Provides deployment on multiple OS from the same directory.

Provides the location of these directories by leveraging the mechanisms defined by
- the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)  specifications on Linux and macOS
- the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows

## Installation
### Cargo
```sh
cargo install rrcm
# initialize config file
rrcm init
# initialize config file from url
rrcm init <url>
```

## Configuration
The configuration file is a yaml file.
configuration file path:
- Unix: $HOME/.config/rrcm/config.yaml
- Win: %PROFILE%\AppData\Roaming\rrcm\config.yaml

### Repository
The repository is defined in the config.yaml file.
```yaml
repos:
# your dotfiles repository
# you can define multiple repositories
  example1: 'git@github:example/example1'
  example2: 'git@github:example/example2'
```
the repository is a directory that contains the dotfiles.
Directory structure example:
```rust
dotfiles
├── home
│   ├── .profile -> $HOME/.profile(Unix)
│   │              %PROFILE%\.profile(Win)
│   └── ...
├── config
│   ├── nushell  -> $HOME/.config/nushell(Unix),
│   │               %PROFILE%\AppData\Roaming\nushell(Win)
│   └── ...
└── config_local
    ├── nvim     -> $HOME/.config/nvim(Unix),
    │               %PROFILE%\AppData\Local\nvim(Win)
    └── ...
```
home, config, config_local are the deployment targets.
- home: Deploy to $HOME(Unix), %PROFILE%(Win)
- config: Deploy to $HOME/.config(Unix), %PROFILE%\AppData\Roaming(Win)
- config_local: Deploy to $HOME/.config/local(Unix), %PROFILE%\AppData\Local(Win)

Under the deployment target, the file or directory is deployed by symbolic link.
**Windows needs to be run as administrator.**

### Deployment target
The deployment target is defined in the config.yaml file.
```yaml
deploy:
  home:
    windows: "%USERPROFILE%"
    mac: "${HOME}"
    linux: "${HOME}"
  config:
    windows: "%FOLDERID_RoamingAppData%"
    mac: "${XDG_CONFIG_HOME}"
    linux: "${XDG_CONFIG_HOME}"
  config_local:
    windows: "%FOLDERID_LocalAppData%"
    mac: "${XDG_CONFIG_HOME}"
    linux: "${XDG_CONFIG_HOME}"
```

Environment variables can be used in the deployment target.
Format
- Unix: ${ENVIRONMENT_VARIABLE_NAME}
- Windows: %ENVIRONMENT_VARIABLE_NAME%

The following special variables are available.
- Unix [XDG base directory](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
    if the environment variable is not set, the default value is used.
    - ${XDG_CONFIG_HOME}
    - ${XDG_DATA_HOME}
    - ${XDG_CACHE_HOME}
- Windows [Known Folder ID](https://docs.microsoft.com/en-us/windows/win32/shell/knownfolderid)
    - %FOLDERID_RoamingAppData%
    - %FOLDERID_LocalAppData%
    - %FOLDERID_Documents%
    - %FOLDERID_Desktop%

## Examples
```sh
# update all repositories
rrcm update

# show status all repositories
rrcm status
```

