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

Default setting (rrcm.toml)
```toml
# deploy path for dotfiles/home
[deploy.home]
windows = '%USERPROFILE%'
mac = '${HOME}'
linux = '${HOME}'

# deploy path for dotfiles/config
[deploy.config]
# windows can use Known Folder
windows = '%FOLDERID_RoamingAppData%'
mac = '${XDG_CONFIG_HOME}'
linux = '${XDG_CONFIG_HOME}'

# deploy path for dotfiles/config_local
[deploy.config_local]
windows = '%FOLDERID_LocalAppData%'
mac = '${XDG_CONFIG_HOME}'
linux = '${XDG_CONFIG_HOME}'

# you can add more directories
```

## Deploy files
```sh
# deploy under dtifles/config
# will create symlync
# on windows, need administrator privilege
rrcm deploy <your dotfiles repo>/config/*
# deploy under dtifles/home
rrcm deploy <your dotfiles repo>/home/*
...
```

On Windows, need administrator privilege.

if you installed [gsudo](https://github.com/gerardog/gsudo).
```sh
sudo rrcm deploy <your dotfiles repo>/config/*
...
```
