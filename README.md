# rrcm
[![Crates.io](https://img.shields.io/crates/v/rrcm.svg)](https://crates.io/crates/rrcm)
[![Workflow Status](https://github.com/mizuki0629/rrcm/workflows/Test/badge.svg)](https://github.com/mizuki0629/rrcm/actions?query=workflow%3A%22Test%22)
[![codecov](https://codecov.io/gh/mizuki0629/rrcm/branch/master/graph/badge.svg?token=IVPHQ5UQIL)](https://codecov.io/gh/mizuki0629/rrcm)

A cross-platform compatible tool for deploying multiple dotfiles repositories.

## Introduction
- Deploy configuration files and directories using symbolic links.
- Configuration files and directories are managed in a git repository.
- Provides deployment on multiple OS from the same directory.

Provides the location of these directories by leveraging the mechanisms defined by
- the [XDG base directory](https://standards.freedesktop.org/basedir-spec/basedir-spec-latest.html)  specifications on Linux and macOS
- the [Known Folder](https://msdn.microsoft.com/en-us/library/windows/desktop/dd378457.aspx) API on Windows

## Direcotry
The configuration file specifies the mapping between deployment sources and destinations for each platform.
rrcm creates symbolic links to files and directories directly under the deployment targets in the deployment destination.
```
(dotfiles repository download directory)
dotiles
│   (local repositroy)
├── example1
│   │   (deploy target)
│   ├── home
│   │   │               (symbolic link)
│   │   ├── .profile <- $HOME/.profile     (on Linux and Mac)
│   │   │               %PROFILE%\.profile (on Win)
│   │   ├── .vim     <- $HOME/.vim         (on Linux and Mac)
│   │   │               %PROFILE%\.vim     (on Win)
│   │   └── ...
│   │
│   └── .config
│       ├── nvim     <- $HOME/.config/nvim                  (on Linux and Mac),
│       │               %PROFILE%\AppData\LocalAppData\nvim (on Win)
│       └── ...
│
└── example2 (next local repositroy)
    └── ...
```
Under the deployment target dierctory, files and directorys are deployed by symbolic link.
**Windows needs to be run as administrator.**


## Configuration
configuration file path:
- Unix: $HOME/.config/rrcm/config.yaml
- Win: %PROFILE%\AppData\Roaming\rrcm\config.yaml
```yaml
---
# dotfiles repositroy download directory
dotfiles:
  windows: "%USERPROFILE%\\dotfiles"
  mac: "${HOME}/.dotfiles"
  linux: "${HOME}/.dotfiles"

# repositories. multiple repositories can be specified.
repos:

    # local repository name
  - name: example1

    # git repository url
    url: 'git@github:example/example1'

    # deploy configuration
    deploy:

      # deploy target
      # Example: deploy home directory to $HOME or %USERPROFILE%
      home:

        # deploy destination on each OS.
        # if OS not defined, it will not be deployed.
        windows: "%USERPROFILE%"
        mac: "${HOME}"
        linux: "${HOME}"

      # Example: deploy .config directory to XDG_CONFIG_HOME or %USERPROFILE%\AppData\Roaming
      .config:
        windows: "%FOLDERID_RoamingAppData%"
        mac: "${XDG_CONFIG_HOME}"
        linux: "${XDG_CONFIG_HOME}"

      # Example: deploy .config-LocalAppData to XDG_CONFIG_HOME or %USERPROFILE%\AppData\Local
      .config-LocalAppData:
        windows: "%FOLDERID_LocalAppData%"
        mac: "${XDG_CONFIG_HOME}"
        linux: "${XDG_CONFIG_HOME}"

    # next repository
  - name: example2
    url: 'git@github:example/example2'
    ...
```

### Deployment destination
Environment variables can be used in deployment destination.

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

## Install
### Cargo
```sh
cargo install rrcm
```

## Usage
initialize config file
```sh
rrcm init
# or initialize config file from http
rrcm init <url>
```
I recommend using gist like this.
[my config.yaml](https://gist.github.com/mizuki0629/1f7e73703b09551610b18392e375bd73)

update(git clone or pull) repositories and deploy
```sh
rrcm update
```

show deploy status
```sh
rrcm status
```

