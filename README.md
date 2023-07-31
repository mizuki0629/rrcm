# rrcm
[![Lint and Test](https://github.com/mizuki0629/rrcm/actions/workflows/rust.yml/badge.svg)](https://github.com/mizuki0629/rrcm/actions/workflows/rust.yml)
 [![Crates.io](https://img.shields.io/crates/v/rrcm.svg)](https://crates.io/crates/rrcm) [![docs.rs](https://docs.rs/rrcm/badge.svg)](https://docs.rs/rrcm/)
 
Rust RC file Management commands.

## Introduction
- Deploy configuration files and directories using symbolic links.
- Provides deployment on multiple OS from the same directory.

Provides the location of these directories by leveraging the mechanisms defined by
- the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)  specifications on Linux and macOS
- the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows

```
dotfiles
├── rrcm.toml       Deploy setting file.
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

## Installation
```sh
cargo install rrcm
```

## Init Configuration
```sh
git clone <your dotfiles repo>
rrcm init <your dotfiles repo>
```

Default setting
```toml
# deploy path for dotfiles/home
[deploy.home]
windows = '%USERPROFILE%'
mac = '${HOME}'
linux = '${HOME}'

# deploy path for dotfiles/config
[deploy.config]
windows = '%FOLDERID_RoamingAppData%'
mac = '${XDG_CONFIG_HOME}'
linux = '${XDG_CONFIG_HOME}'

# deploy path for dotfiles/config_local
[deploy.config_local]
windows = '%FOLDERID_LocalAppData%'
mac = '${XDG_CONFIG_HOME}'
linux = '${XDG_CONFIG_HOME}'

# you can define more directories
```

## Deploy files
```sh
# deploy under dtifles/config
rrcm deploy <your dotfiles repo>/config/*
rrcm deploy <your dotfiles repo>/home/*
...
```
