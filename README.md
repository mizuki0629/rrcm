# rrcm
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
├── home         -> $HOME(Linux, macOS),
│   │               %PROFILE%\AppData\Local(Windows)
│   └── .profile
├── config       -> $HOME/.config(Linux, macOS),
│   │               %PROFILE%\AppData\Roaming(Windows)
│   ├── fish
│   └── tmux 
└── config_local -> $HOME/.config(Linux, macOS),
    │               %PROFILE%\AppData\Local(Windows)
    └── nvim
```

## Installation
```sh
cargo install rrcm
```

## Init Configuration
```sh
rrcm init <path> # dotfiles directory
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
rrcm deploy dotfiles/config/*
rrcm deploy dotfiles/home/*
...
```
