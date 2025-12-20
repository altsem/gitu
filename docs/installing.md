## Installing Gitu
### Using Cargo
Run the command (recommended):
```
cargo install gitu --locked
```

...or to install from git, run:
```
cargo install --git https://github.com/altsem/gitu.git --locked
```

### Arch Linux
You can install the `gitu` package from the [official extra repository](https://archlinux.org/packages/extra/x86_64/gitu/):

```
pacman -S gitu
```

### Using Release binaries
gitu is available on Github Releases and should be installed from there.

The latest release is available
[here](https://github.com/altsem/gitu/releases).

Download the archive that is appropriate for your platform and extract the
binary into your `$PATH`. A common valid path location is `/usr/local/bin`.

### Using Mise

You can use [mise-en-place](https://github.com/jdx/mise), a polyglot tool version manager to install and make available for all your projects the last version of gitu using a command like:

```shell
mise use -g gitu@latest
```

### Using Nix flakes
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


