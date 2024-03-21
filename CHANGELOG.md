# Changelog

All notable changes to this project will be documented in this file.

## [0.7.2] - 2024-03-16

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

