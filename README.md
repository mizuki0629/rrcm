# rrcm
[![Crates.io](https://img.shields.io/crates/v/rrcm.svg)](https://crates.io/crates/rrcm)
[![Workflow Status](https://github.com/mizuki0629/rrcm/workflows/main/badge.svg)](https://github.com/mizuki0629/rrcm/actions?query=workflow%3A%22main%22)

### Introduction
- Deploy configuration files and directories using symbolic links.
- Configuration files and directories are managed in a git repository.
- Provides deployment on multiple OS from the same directory.

Provides the location of these directories by leveraging the mechanisms defined by
- the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)  specifications on Linux and macOS
- the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows

### Installation
#### Cargo
```sh
cargo install rrcm
```

### Configuration
The configuration file is a TOML file.
configuration file path:
- Unix: $HOME/.config/rrcm/config.toml
- Win: %PROFILE%\AppData\Roaming\rrcm\config.toml

#### Repository
The repository is defined in the config.toml file.
```toml
[repos]
# your dotfiles repository
# you can define multiple repositories
example1 = "git@github:example/example1"
example2 = "git@github:example/example2"
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

#### Deployment target
The deployment target is defined in the config.toml file.
```toml
[deploy.config]
windows = '%FOLDERID_RoamingAppData%'
mac = '${XDG_CONFIG_HOME}'
linux = '${XDG_CONFIG_HOME}'

[deploy.config_local]
windows = '%FOLDERID_LocalAppData%'
mac = '${XDG_CONFIG_HOME}'
linux = '${XDG_CONFIG_HOME}'

[deploy.home]
windows = '%USERPROFILE%'
mac = '${HOME}'
linux = '${HOME}'
```

#### Environment variables
The following environment variables are available.
Special variables are available on each OS.

- Windows [Known Folder ID](https://docs.microsoft.com/en-us/windows/win32/shell/knownfolderid)
    - %FOLDERID_RoamingAppData%
    - %FOLDERID_LocalAppData%
    - %FOLDERID_Documents%
    - %FOLDERID_Desktop%

- Unix [XDG base directory](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
    - $XDG_CONFIG_HOME
    - $XDG_DATA_HOME
    - $XDG_CACHE_HOME
    - $XDG_RUNTIME_DIR

### Examples
```sh
# update all repositories
rrcm update

# show status all repositories
rrcm status
```


License: MIT
