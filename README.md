# Gitu
## It's Gitu - A git porcelain outside of Emacs

A stand-alone terminal user interface for Git. Heavily inspired by Magit.

<img src="doc/gitu.png" width="600" />

## Features
Gitu aims to implement many of the core features of Magit over time. 
It should be familiar to any previous Magit users.

<img src="doc/help.png" width="600" />

## CLI
Gitu can drop you right into a log, or to the display of a commit:
```
gitu log <git_log_args>
gitu show <git_show_args>
```

## Install
### Using Cargo
Clone the repo and run:
`cargo install --path . --locked`

## Development
### Benchmarking
`cargo bench`

### Profiling
This project comes with pprof as a dev-dependency. You can run it with:
`cargo bench --bench show -- --profile-time 5`

A flamegraph would then be output to `target/criterion/show/profile/flamegraph.svg`
