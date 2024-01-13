# gitu
An editor-agnostic git TUI heavily inspired by Magit.

## Dependencies
Requires `git` and `delta` to be on your PATH.
`delta` is used for formatting diff output.

## Install
### Using Cargo
Clone the repo and run:
`cargo install --path .`

## Hotkeys (WIP)
- q - Quit
- g - Refresh items
- TAB - Toggle section

- j/k - Move down/up
- C-d/C-u - Scroll half-page down/up

- l - Go to log screen

- s - Stage / Apply
- u - Unstage / Apply in reverse

- c - git commit
- f - git fetch --all

## Features
- [ ] Staging / Unstaging (and apply / reverse)
  - [ ] Whole sections (Unstaged, Staged, Untracked)
  - [x] Files
  - [x] Hunks
  - [ ] Line-by-line
- [x] Toggle sections
- [x] Push / pull
- [x] Scrolling
- [x] Colorized / highlighted diffs
- [x] Open in editor
- [ ] Magit-like stateful hotkeys (c -> a to amend commit)
  - [ ] Command args e.g. --force on push
  - [ ] Command-palette to visualize hotkeys hotkeys
- [ ] Run arbitrary git command
- [ ] Line wrapping
