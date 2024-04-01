# Changelog

All notable changes to this project will be documented in this file.

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

