## Development & Tooling

Here's a place to share and document some tooling in use by Gitu, as well as tooling you may want to use.

### CI 

The CI is mostly running a script which you may also run locally to get a faster feedback loop.
Install the tools required and run it if you wish. It's located at [ci.sh](ci.sh).
I typically use it along with [entr](https://github.com/eradman/entr) to watch for changes:
```bash
rg -l . | entr -c ./ci.sh
```

### Testing

Gitu makes heavy use of snapshot-testing with a library called **Insta** (https://insta.rs/).
Most tests are written on a pretty high level.
The philosophy is to keep the tests easy to reason about, and make refactoring painless.

### Changelog

The changelog is generated automatically using **git-cliff** (https://git-cliff.org/).
There's no strict commit convention, but using certain formats will create a new entry.
You can see which commit formats contribute to a changelog in [cliff.toml](cliff.toml)

### Releases

In order to create a release (by a maintainer).
Run the [bump.sh](bump.sh) script. This will create a commit with appropriate changes
to prepare for a release and tag it with a version.
Push this commit (and tag) to **master** in order to kick off the release Github Actions.
