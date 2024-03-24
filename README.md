## It's Gitu! - A Git porcelain *outside* of Emacs
[![CI](https://github.com/altsem/gitu/actions/workflows/ci.yml/badge.svg)](https://github.com/altsem/gitu/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/altsem/gitu/graph/badge.svg?token=5YWPU7GWFW)](https://codecov.io/gh/altsem/gitu)

A terminal user interface for Git. Inspired by Magit, and launched straight from the terminal.

<img src="vhs/rec.gif"/>

### Features
Gitu aims to implement many of the core features of Magit over time. 
It should be familiar to any previous Magit users.

A rough list of so-far supported features:
- File/Hunk/Line-level stage/unstage
- Show (view commits / open EDITOR at line)
- Show branches
- Branch:
  - checkout, checkout new
- Commit:
  - commit, amend, fixup
- Fetch:
  - all
- Log:
  - current, other
- Pull / Push:
  - remote
- Rebase:
  - abort, continue, autosquash, interactive
- Reset:
  - Soft, Mixed, Hard
- Stash:
  - save, pop, apply, drop

### Keybinds
A help-menu can be shown by pressing the `h` key.

<img src="vhs/help.png"/>

### Install
#### Using Cargo
Run the command (recommended):
```
cargo install gitu --locked
```

...or to install from git, run:
```
cargo install --git https://github.com/altsem/gitu.git --locked
```

#### Arch Linux
You can install the `gitu` package from the [official extra repository](https://archlinux.org/packages/extra/x86_64/gitu/):

```
pacman -S gitu
```

#### Using Release binaries
gitu is available on Github Releases and should be installed from there.

The latest release is available
[here](https://github.com/altsem/gitu/releases).

Download the archive that is appropriate for your platform and extract the
binary into your `$PATH`. A common valid path location is `/usr/local/bin`.

#### Using Nix flakes
To build from `master` on flaked Nix platforms add this repo to your inputs:

```nix
inputs = {
  nixpkgs.url = "nixpkgs/nixos-unstable";
  gitu.url = "github:altsem/gitu";
};
```

Then wherever you install your packages (i.e., `home-manager`):

```nix
{ inputs, pkgs, ... }: {
  home.packages = [ inputs.gitu.packages.${pkgs.system}.default ];
}
```

You can also use this repo's binary cache to avoid building gitu:

```nix
nix.settings = {
  extra-substituters = [ "https://gitu.cachix.org" ];
  extra-trusted-public-keys =
    [ "gitu.cachix.org-1:iUIaNys1l3W1LF/M8OXzaTl7N/OinGOlzdUJUSc+5eY=" ];
}
```

### Configuration
The environment variables `GIT_EDITOR`, `VISUAL` or `EDITOR` (checked in this order) dictate which editor Gitu will open.

Configuration is also loaded from `~/.config/gitu/config.toml`,
you could copy the [default configuration](src/default_config.toml).
