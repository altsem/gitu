# Gitu
A git TUI heavily inspired by Magit.

<img src="doc/gitu.png" width="600" />

## Install
### Using Cargo
Clone the repo and run:
`cargo install --path . --locked`

## Hotkeys (WIP)
| Key     | Action                     |
| ------- | -------------------------- |
| q       | Quit                       |
| g       | Refresh items              |
| y       | Copy to clipboard          |
| TAB     | Toggle section             |
| j/k     | Move down/up               |
| C-d/C-u | Scroll half-page down/up   |
| l       | Go to log screen           |
| s       | Stage / Apply              |
| u       | Unstage / Apply in reverse |
| c       | git commit                 |
| f       | git fetch --all            |

## CLI
Gitu can drop you right into a log, or to the display of a commit:
```
gitu log <git_log_args>
gitu show <git_show_args>
```

## Development
### Benchmarking
`cargo bench`

### Profiling
This project comes with ppref as a dev-dependency. You can run it with:
`cargo bench --bench show -- --profile-time 5`

A flamegraph would then be output to `target/criterion/show/profile/flamegraph.svg`
