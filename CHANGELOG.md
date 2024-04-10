# Changelog

All notable changes to this project will be documented in this file.

## [0.15.0] - 2024-04-10

### 🚀 Features

- Config option: general.always_show_help.enabled
- Add all args to rebase menu
- Add --prune and --tags flags to Fetch menu
- Add all on/off arg flags to Commit menu
- Add --rebase pull arg
- Add --force /--no-verify /--dry-run push args

### 🐛 Bug Fixes

- Main screen is more smart about scrolling when menu is open
- Redraw screen even if command failed
- Only stderr would show in log popup

### 🎨 Styling

- Display args more like Magit

## [0.14.0] - 2024-04-06

### 🚀 Features

- Remove move p/n from default bindings (move up/down)
- Make keybinds configurable

### 🐛 Bug Fixes

- Existing terminal text would bleed into gitu on startup
- Discarding staged files would not work & use git clean for removing untracked files

## [0.13.1] - 2024-04-04

### 🐛 Bug Fixes

- Handle EDITOR args, and better deal with absolute paths

## [0.13.0] - 2024-04-04

### 🚀 Features

- Support sending keys on startup with a cli flag (-k)

### 🐛 Bug Fixes

- Prompt stash action instead of always "Stash index"
- Edge cases and error handling for stashing worktree (#103)

### 🎨 Styling

- Improve menu layout and define new keybind display

## [0.12.1] - 2024-04-01

### 🐛 Bug Fixes

- Resolve issue showing files with crlf

## [0.12.0] - 2024-04-01

### 🚀 Features

- Set '--jobs' to 10 when running 'git fetch'
- Run fetch, pull and push without blocking the ui
- Implement 'rebase elsewhere', it prompts you where to rebase
- Prompt for rev on reset soft/mixed/hard
- Show multiple command outputs in popup

### 🐛 Bug Fixes

- Improve error-handling of external commands
- Discarding staged files wouldn't work

### 🎨 Styling

- Change command popup to be more intuitive

## [0.11.0] - 2024-03-27

### 🚀 Features

- Togglable argument '--force-with-lease' when pushing

### 🎨 Styling

- Show quit/close keybind on all menus

## [0.10.0] - 2024-03-24

### 🚀 Features

- Prompt input rev for 'log other'

### 🎨 Styling

- Style.selection_area now includes cursor line
- Change cursor to a vertical bar, add config `style.cursor`

## [0.9.1] - 2024-03-23

### 🐛 Bug Fixes

- Crash when trying to show diff of binary files

## [0.9.0] - 2024-03-23

### 🚀 Features

- Unstage individual lines with ctrl-up/down and 'u'
- Stage individual lines with ctrl-up/down and 's'
- Add configurable quit confirmation

### 🐛 Bug Fixes

- Cursor now skips unselectable lines more deterministically

## [0.8.0] - 2024-03-22

### 🚀 Features

- Show stash status, add 'save', 'pop', 'apply' and 'drop' actions
- Unstage all staged changes by hovering 'Staged' section
- Stage all unstaged changes by hover 'Unstaged' section
- Stage all untracked files by hovering 'Untracked' section

## [0.7.0] - 2024-03-16

### 🚀 Features

- Add --version flag
- Add Nix flake via ipetkov/crane

### 🐛 Bug Fixes

- Crate would not build (due to trying to get version via git)

## [0.6.3] - 2024-03-13

### 🐛 Bug Fixes

- Interactive rebase includes parent (like magit)
- Target binds in help-menu had wrong name formatting

## [0.6.2] - 2024-03-12

### 🐛 Bug Fixes

- Include changelog entry in github release

## [0.6.1] - 2024-03-12

### 🐛 Bug Fixes

- Release to Github

## [0.6.0] - 2024-03-12

### 🚀 Features

- Prompt what to checkout, default to selected item (like Magit)

## [0.5.5] - 2024-03-11

### 🐛 Bug Fixes

- Gitu would not open inside submodules

## [0.5.4] - 2024-03-10

### 🐛 Bug Fixes

- Fixed scrolling after breaking in previous update

## [0.5.3] - 2024-03-09

### 🐛 Bug Fixes

- Rebase --continue freeze

## [0.5.2] - 2024-03-08

### 🐛 Bug Fixes

- Cursor would disappear when staging the last hunk of a delta
- Issue when cursor would disappear after external git updates

### 🎨 Styling

- Remove trailing space in 'Create and checkout branch: '

## [0.5.1] - 2024-03-07

### 🐛 Bug Fixes

- Would not start on windows due to nix signal handling

## [0.5.0] - 2024-03-07

### 🚀 Features

- Move 'reset' keybind to capital 'X' to mimic magit
- Proper y/n prompt when discarding things

### 🐛 Bug Fixes

- Annotated tags would not display

## [0.4.0] - 2024-03-06

### 🚀 Features

- Add `style.line_highlight.[un]changed` config options

### 🐛 Bug Fixes

- Terminal would corrupt text when quitting gitu after opening editor
- Terminal would corrupt text when gitu crashed

## [0.3.0] - 2024-03-05

### 🚀 Features

- Read not just EDITOR env var, but GIT_EDITOR & VISUAL too
- Add error popup and more graceful error handling
- Improve CHANGELOG.md format
- Replace --exit-immediately cli flag with new --print

### 🐛 Bug Fixes

- Show author date (not commit date) on commits like 'git log'

### 🎨 Styling

- Selection_line & selection_area now extend fully to left

