# Changelog

All notable changes to this project will be documented in this file.

## [0.38.0] - 2025-10-10

### 🚀 Features

- Don't refresh previous screen when closing a nested one
- Revert to using Crossterm as backend (fixes rendering/input bugs)

### ⚡ Performance

- Revert back to forking out to git to check status (faster)
- Skip status check if `status.showUntrackedFiles false`, helps in large repos
- Avoid excessive allocation while computing hunk highlights
- Change rendering output from Stderr to Stdout - more efficient

## [0.37.0] - 2025-09-28

### 🚀 Features

- Support file line positions for the micro editor (#424)
- Add mouse wheel scrolling support
- Simple mouse interactions
- Show the diff on the stash detail screen

### 🐛 Bug Fixes

- Mouse clicks on invalid screen lines trigger actions
- Avoid redrawing for unhandled mouse events
- Disable mouse reporting when mouse support is disabled
- Workaround Termwiz mouse scroll event buggy handling

## [0.36.0] - 2025-09-16

### 🚀 Features

- Allow configuring recent commits and stash list limits
- *(config)* Removed support of sequences of keys (e.g. abc)

### 🐛 Bug Fixes

- *(config)* Report invalid key binding errors
- Process bug when running show commands on Windows (#330)

## [0.35.0] - 2025-09-06

### 🚀 Features

- Ability to invoke merge operations (#401)
- Add config cli arg to override the config file to use (#400)

### 🐛 Bug Fixes

- Crash when opening PHP files (#405)

## [0.34.0] - 2025-06-29

### 🚀 Features

- Ability to delete a remote
- Commit extend (#396)
- Optimize & defer rendering of items in editor, esp. diff hunk highlights (#392)
- Ability to rename a remote
- Switch terminal backend from Crossterm to Termwiz

### 🐛 Bug Fixes

- Shift modifier & uppercase key events would not work in certain terminals (#395)

### ⚡ Performance

- More efficiently keep track of changes between updates

## [0.33.0] - 2025-06-07

### 🚀 Features

- Discard line by line (+ configure when to confirm discard)
- Ability to add a remote

### 🐛 Bug Fixes

- *(highlighting)* Update tree-sitter, replace dated toml lib with toml-ng
- *(show)* Crash when sometimes attempting to show a commit
- Opening hunk in EDITOR used wrong line number
- Unmerged branches could not be deleted via discard action

## [0.32.0] - 2025-05-24

### 🚀 Features

- Hint external commands to output colors
- Add delete option to branch menu

### 🐛 Bug Fixes

- Ignore diff.external, in case its set to an unsupported tool #369
- *(prompt)* Freeze when a command would fail after a prompt occurred

## [0.31.0] - 2025-05-05

### 🚀 Features

- Disable filewatcher when `status.showUntrackedFiles` is off
- Replace `ignore` lib with `libgit2` ignore functionality
- FileWatcher now ignores changes from patterns in .gitignore

### 🐛 Bug Fixes

- Support custom path prefixes in Git diff parser (e.g. i/... w/...) (#361)
- Disable filewatcher when it fails to initialize

## [0.30.3] - 2025-04-21

### 🐛 Bug Fixes

- *(crates-io-release)* Resolve issue with publishing to crates-io

## [0.30.2] - 2025-04-21

### 🐛 Bug Fixes

- *(crates-io-release)* Specify gitu-diff to not be published

## [0.30.1] - 2025-04-21

### 🐛 Bug Fixes

- Issue with project lockfile/release

## [0.30.0] - 2025-04-21

### 🚀 Features

- Print command stderr to screen as they run (e.g. git hooks)
- Fall back to remote.pushDefault when branch pushRemote is not set
- `GITU_SHOW_EDITOR` env var as an option above `EDITOR` etc.
- Improve on error-handling. Errors should now provide more context.
- Change "conflicted" file status to "unmerged", remove redundant "unmerged" section
- New diff-parser, easier to maintain, integrates better

### 🐛 Bug Fixes

- *(file-watcher)* Freeze on startup, log error and stop on failure
- Accurate --version in Github releases

## [0.29.0] - 2025-03-10

### 🚀 Features

- Add support for `nvr` command with line number navigation

## [0.28.2] - 2025-02-19

### 🐛 Bug Fixes

- Rebase menu opening after closing Neovim

## [0.28.1] - 2025-02-13

### 🐛 Bug Fixes

- Change logging level to reduce inotify spam
- Don't refresh on `gitu.log` writes (gitu --log)

## [0.28.0] - 2025-02-04

### 🚀 Features

- *(Revert)* Add --no-edit flag (bound to -E)
- Update on file changes
- Open help with `?` too, close with `h` / `?` (#280)

### 🐛 Bug Fixes

- Cursor sometimes hidden when spawning editor
- 'Standard input is not a terminal' when opening editor
- Staircased git output

### 🔧 Configuration

- Add `refresh_on_file_change` bool to en/disable file watcher

## [0.27.0] - 2024-11-05

### 🚀 Features

- *(config)* Collapse screen headers via e.g. `general.collapsed_sections = ["recent_commits"]`

### 🐛 Bug Fixes

- Set version properly in Github release

## [0.26.0] - 2024-10-24

### 🚀 Features

- Support "The Two Remotes": https://magit.vc/manual/3.2.0/magit/The-Two-Remotes.html

### 🔧 Configuration

- Bind 'Pu' to new action: `push_to_upstream`
- Bind 'Pp' to new action: `push_to_push_remote`
- Bind 'Fu' to new action: `pull_from_upstream`
- Bind 'Fp' to new action: `pull_from_push_remote`
- Remove 'Pp' <-> `git push` (depended on `push.default`)
- Remove 'Fp' <-> `git pull` (from upstream)
- Rename `push_elsewhere` to `push_to_elsewhere`
- Rename `pull_elsewhere` to `pull_from_elsewhere`

## [0.25.0] - 2024-09-03

### 🚀 Features

- Change priority order of editor envvar lookup

## [0.24.0] - 2024-08-05

### 🚀 Features

- *(status)* Detect renamed files

## [0.23.1] - 2024-07-23

### 🐛 Bug Fixes

- *(instant fixup commit)* Would not work with some versions of Git
- *(instant fixup commit)* Use --keep-empty and --autostash like Magit
- *(instant fixup commit)* Errors wouldn't show

## [0.23.0] - 2024-07-18

### 🚀 Features

- *(commit)* Instant fixup

### 🐛 Bug Fixes

- Invisible menu after closing an input prompt

### 🎨 Styling

- Wording in menus made more consistent to Magit

## [0.22.1] - 2024-07-04

### 🐛 Bug Fixes

- Upgrade libgit2 to 1.8.1 to support new `index.skipHash` git config

## [0.22.0] - 2024-06-27

### 🚀 Features

- Make cursor and selection symbol configurable

## [0.21.1] - 2024-06-19

### 🐛 Bug Fixes

- Tabs would not be rendered, render them as 4 spaces for now
- *(flake)* Add AppKit to build inputs

## [0.21.0] - 2024-06-16

### 🚀 Features

- -n argument to limit log
- -F to grep for commits in log menu
- Support value arguments

### 🐛 Bug Fixes

- Pin exact tree-sitter version to prevent common build breaks

## [0.20.1] - 2024-05-08

### 🐛 Bug Fixes

- Bad diffs when git's `autocrlf` was enabled

## [0.20.0] - 2024-05-08

### 🚀 Features

- Add "elsewhere" option to fetch, pull and push menu
- Syntax highlighting for Elixir

## [0.19.2] - 2024-04-25

### 🐛 Bug Fixes

- Hint/preserve missing newlines in diffs/patches

## [0.19.1] - 2024-04-21

### 🐛 Bug Fixes

- Crash when trying to highlight `.tsx` files

## [0.19.0] - 2024-04-21

### 🚀 Features

- Move to parent section with alt+h
- Move to next/prev sections with alt+j and alt+k
- On MacOS: load `~/.config/gitu/config.toml` instead of `~/Library/Application Support/gitu/config.toml`
- Add Revert commit/abort/continue
- Show revert status

### 🐛 Bug Fixes

- Scala syntax highlighter would not load

## [0.18.4] - 2024-04-20

### 🐛 Bug Fixes

- *(ci)* Release dir would not be created

## [0.18.3] - 2024-04-20

### 🐛 Bug Fixes

- Release to windows

## [0.18.0] - 2024-04-20

### 🚀 Features

- Syntax highlighting with tree-sitter and revamp of diff style config

### 🐛 Bug Fixes

- *(log)* Ignore `prefetch/remotes/` refs

## [0.17.1] - 2024-04-17

### 🐛 Bug Fixes

- Moving page up/down resulted in view being refreshed

## [0.17.0] - 2024-04-17

### 🚀 Features

- Log whether config file is being loaded or not on startup (--log flag)
- Add blank lines between refs sections, don't show empty sections
- Segregate remotes into separate sections
- Make 3 sections in show refs screen: branches, remotes, tags

### 🐛 Bug Fixes

- When head detached show "?" instead of "*" on target match; update tests

## [0.16.0] - 2024-04-14

### 🚀 Features

- Copy commit hash with "y", move Show Refs to "Y"
- Cursor is kept in view when scrolling
- Togglable stash flags: --all & --include-untracked

### 🐛 Bug Fixes

- Typo in descriptions on menu

### 🎨 Styling

- Update stash promps to be more like in Magit

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

## [0.2.0] - 2024-03-04

<!-- generated by git-cliff -->
