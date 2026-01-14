## It's Gitu! - A Git porcelain *outside* of Emacs
[![CI](https://github.com/altsem/gitu/actions/workflows/ci.yml/badge.svg)](https://github.com/altsem/gitu/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/altsem/gitu/graph/badge.svg?token=5YWPU7GWFW)](https://codecov.io/gh/altsem/gitu)

A terminal user interface for Git. Inspired by Magit.

<img style="width: 720px" src="vhs/rec.gif"/>

### Features
Gitu aims to implement many of the core features of Magit over time.
It should be familiar to any previous Magit users.\
Here's a list of so-far supported features:
- **Staging/Unstaging** _(file, hunk, line)_ 
- **Showing** _(view commits / open EDITOR at line)_
- **Branching** _(checkout, checkout new)_
- **Committing** _(commit, amend, fixup)_
- **Fetching**
- **Logging** _(current, other)_
- **Pulling / Pushing** _to/from configured upstream/pushDefault_
- **Rebasing** _(elsewhere, abort, continue, autosquash, interactive)_
- **Resetting** _(soft, mixed, hard)_
- **Reverting** _(commit)_
- **Stashing** _(save, pop, apply, drop)_

### Keybinds
Keybinds try mimic Magit, while staying Vim-like.
A help-menu can be shown by pressing the `h` key, or by configuring `general.always_show_help.enabled = true`


<img style="width: 720px" src="vhs/help.png"/>

### Configuration
The environment variables `VISUAL`, `EDITOR` or `GIT_EDITOR` (checked in this order) dictate which editor Gitu will open. This means that e. g. commit messages will be opened in the `GIT_EDITOR` by Git, but if the user wishes to do edits to the actual files in a different editor, `VISUAL` or `EDITOR` can be set accordingly.

Configuration is also loaded from:
- Linux:   `~/.config/gitu/config.toml`
- macOS:   `~/.config/gitu/config.toml`
- Windows: `%USERPROFILE%\AppData\Roaming\gitu\config.toml`

, refer to the [default configuration](src/default_config.toml).

#### Picker Style Customization

You can customize the appearance of the interactive picker by adding the following to your config:

```toml
[style.picker]
prompt = { fg = "cyan" }                    # Prompt text color
info = { mods = "DIM" }                     # Status line style (e.g., "3/10 matches")
selection_line = { mods = "BOLD" }               # Selected item style
matched = { fg = "yellow", mods = "BOLD" }  # Fuzzy-matched characters highlight
```

### Installing Gitu
Follow the install instructions: [Installing Gitu](docs/installing.md)\
Or install from your package manager:

[![Packaging status](https://repology.org/badge/vertical-allrepos/gitu.svg)](https://repology.org/project/gitu/versions)

### Contributing
PRs are welcome!
This may help to get you started: [Development & Tooling](docs/dev-tooling.md)
