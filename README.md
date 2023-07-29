# rrcm
Rust RC file Management commands.

## Introduction
- Deploy configuration files and directories using symbolic links.
- Provides deployment on multiple OS from the same directory.

Provides the location of these directories by leveraging the mechanisms defined by

- the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html) and the [XDG user directory](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/) specifications on Linux
- the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows
- the [Standard Directories](https://developer.apple.com/library/content/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/FileSystemOverview/FileSystemOverview.html#//apple_ref/doc/uid/TP40010672-CH2-SW6) guidelines on macOS


dotfiles
├── config -> $HOME/.config(Linux, macOS), AppData\Roaming(Windows)
│   ├── fish
│   └── tmux 
├── config_local -> $HOME/.config(Linux, macOS), AppData\Local(Windows)
│   └── nvim
└── home -> $HOME(Linux, macOS, Windows)
    └── .profile


## Installation
```sh
cargo install rrcm
```
## Configuration
```yaml
---
# dotfiles repo path
dotfiles:
  windows:
    # {FOLDERID_Profile}\dotfiles
    Relative:
      base: FOLDERID_Profile
      path: dotfiles
  mac:
    # $HOME/dotfiles
    Relative:
      base: HOME
      path: dotfiles
  linux:
    # $HOME/dotfiles
    Relative:
      base: HOME
      path: dotfiles
# deploy path
deploy:
  home:
    windows:
    # dotfiles/home -> {FOLDERID_Profile}
      Relative:
        base: FOLDERID_Profile
        path: ~
    mac:
    # dotfiles/home -> $HOME
      Relative:
        base: HOME
        path: ~
    linux:
    # dotfiles/home -> $HOME
      Relative:
        base: HOME
        path: ~
  config_local:
    windows:
    # dotfiles/home -> {FOLDERID_Profile}
      Relative:
        base: FOLDERID_LocalAppData
        path: ~
    mac:
      Relative:
        base: Preference
        path: ~
    linux:
      Relative:
        base: XDG_CONFIG_HOME
        path: ~
  config:
    windows:
      Relative:
        base: FOLDERID_RoamingAppData
        path: ~
    mac:
      Relative:
        base: Preference
        path: ~
    linux:
      Relative:
        base: XDG_CONFIG_HOME
        path: ~
```

