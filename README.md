## It's Gitu! - A Git porcelain *outside* of Emacs
[![CI](https://github.com/altsem/gitu/actions/workflows/ci.yml/badge.svg)](https://github.com/altsem/gitu/actions/workflows/ci.yml)

A terminal user interface for Git. Heavily inspired by Magit.
Gitu launches launches straight from the terminal.

<img src="doc/gitu.png" width="600" />

### Features
Gitu aims to implement many of the core features of Magit over time. 
It should be familiar to any previous Magit users.

A rough list of so-far supported features:
- File/Hunk-level stage/unstage
- Show (view commits / open EDITOR at line)
- Show branches
- Branch:
  - checkout
- Commit:
  - commit, amend, fixup
- Fetch:
  - all
- Log:
  - current
- Pull / Push:
  - remote
- Rebase:
  - abort, continue, autosquash, interactive

### Keybinds
A help-menu can be shown by pressing the `h` key.

<img src="doc/help.png" width="400" />

### CLI
Gitu can drop you right into a log, or to the display of a commit:
```
gitu log <git_log_args>
gitu show <git_show_args>
```

### Install
#### Using Cargo
Clone the repo and run:
`cargo install --path . --locked`

### Development
#### Benchmarking
`cargo bench`

#### Profiling
This project comes with pprof as a dev-dependency. You can run it with:
`cargo bench --bench show -- --profile-time 5`

A flamegraph would then be output to `target/criterion/show/profile/flamegraph.svg`
