pub mod app;
mod bindings;
pub mod cli;
mod cmd_log;
pub mod config;
pub mod error;
mod file_watcher;
mod git;
pub mod gitu_diff;
mod highlight;
mod item_data;
mod items;
mod key_parser;
mod menu;
mod ops;
mod prompt;
mod screen;
mod syntax_parser;
pub mod term;
#[cfg(test)]
mod tests;
mod ui;

use bindings::Bindings;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers};
use error::Error;
use git2::Repository;
use items::Item;
use ops::Action;
use std::{
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::Arc,
    time::Duration,
};
use term::Term;

use crate::config::Config;

pub const LOG_FILE_NAME: &str = "gitu.log";

//                                An overview of Gitu's ui and terminology:
//
//                Screen (src/screen/*)
//                  │
//                  ▼
//                 ┌──────────────────────────────────────────────────────────────────┐
//        Item───┬─► On branch master                                                 │
//        Item   └─► Your branch is up to date with 'origin/master'.                  │
//        ...      │                                                                  │
//                 │ Untracked files                                                  │
//                 │ src/tests/rebase.rs                                              │
//                 │                                                                  │
//                 │ Unstaged changes (4)                                             │
//                 │▌modified   src/keybinds.rs…                                      │
//                 │ modified   src/ops/mod.rs…                                       │
//                 │ modified   src/ops/rebase.rs…                                    │
//                 │ modified   src/tests/mod.rs…                                     │
//                 │                                                                  │
//                 │ Stashes                                                          │
//                 │ stash@0 On master: scroll                                        │
//                 │ stash@1 WIP on fix/run-cmd-error-on-bad-exit: 098d14a feat: prom…│
//                 │                                                                  │
//                 │ Recent commits                                                   │
// Ops (src/ops/*) ├──────────────────────────────────────────────────────────────────┤
//       │         │Help                        Submenu      modified   src/keybinds.r│
//       └─────┬───►g Refresh                   h Help       ret Show                 │
//             └───►tab Toggle section          b Branch     K Discard                │
//                 │k p ↑ Move up               c Commit     s Stage                  │
//                 │j n ↓ Move down             f Fetch      u Unstage                │
// Submenu ───────►│C-k C-p C-↑ Move up line    l Log                                 │
//                 │C-j C-n C-↓ Move down line  F Pull                                │
//                 │C-u Half page up            P Push                                │
//                 │C-d Half page down          r Rebase                              │
//                 │y Show refs                 X Reset                               │
//                 │                            z Stash                               │
//                 └──────────────────────────────────────────────────────────────────┘

pub type Res<T> = Result<T, Error>;

pub fn run(config: Arc<Config>, args: &cli::Args, term: &mut Term) -> Res<()> {
    let dir = find_git_dir()?;
    let repo = open_repo(&dir)?;

    let mut app = app::App::create(
        Rc::new(repo),
        term.size().map_err(Error::Term)?,
        args,
        config,
        true,
    )?;

    if let Some(keys_string) = &args.keys {
        let ("", keys) = key_parser::parse_keys(keys_string).expect("Couldn't parse keys") else {
            panic!("Couldn't parse keys");
        };

        for event in keys_to_events(&keys) {
            app.handle_event(term, event)?;
        }
    }

    app.redraw_now(term)?;

    if args.print {
        return Ok(());
    }

    app.run(term, Duration::from_millis(100))?;

    Ok(())
}

fn open_repo(dir: &Path) -> Res<Repository> {
    log::debug!("Opening repo");
    let repo = open_repo_from_env()?;
    repo.set_workdir(dir, false).map_err(Error::OpenRepo)?;
    Ok(repo)
}

fn find_git_dir() -> Res<PathBuf> {
    log::debug!("Finding git dir");
    let dir = PathBuf::from(
        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()
                .map_err(Error::FindGitDir)?
                .stdout,
        )
        .map_err(Error::GitDirUtf8)?
        .trim_end(),
    );
    Ok(dir)
}

fn open_repo_from_env() -> Res<Repository> {
    match Repository::open_from_env() {
        Ok(repo) => Ok(repo),
        Err(err) => Err(Error::OpenRepo(err)),
    }
}

fn keys_to_events(keys: &[(KeyModifiers, KeyCode)]) -> Vec<Event> {
    keys.iter()
        .map(|(mods, key)| {
            Event::Key(KeyEvent {
                code: *key,
                modifiers: *mods,
                kind: event::KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
        })
        .collect::<Vec<_>>()
}
