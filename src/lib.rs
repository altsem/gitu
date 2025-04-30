mod bindings;
pub mod cli;
mod cmd_log;
pub mod config;
pub mod error;
mod file_watcher;
mod git;
mod git2_opts;
pub mod gitu_diff;
mod highlight;
mod items;
mod key_parser;
mod menu;
mod ops;
mod prompt;
mod screen;
pub mod state;
mod syntax_parser;
pub mod term;
#[cfg(test)]
mod tests;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers};
use error::Error;
use file_watcher::FileWatcher;
use git2::Repository;
use items::Item;
use ops::Action;
use std::{
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    time::Duration,
};
use term::Term;

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

#[derive(Debug)]
pub enum GituEvent {
    Term(Event),
    FileUpdate,
}

pub fn run(args: &cli::Args, term: &mut Term) -> Res<()> {
    let dir = find_git_dir()?;
    let repo = open_repo(&dir)?;
    let config = Rc::new(config::init_config()?);

    let watcher = if config.general.refresh_on_file_change.enabled {
        Some(Rc::new(FileWatcher::new(&dir)?))
    } else {
        None
    };

    let get_next_event = move || loop {
        if let Some(event) = poll_term_event() {
            return event;
        } else if let Some(event) = poll_file_watcher(watcher.clone()) {
            return event;
        }
    };

    let mut state = state::State::create(
        Box::new(get_next_event),
        Rc::new(repo),
        term.size().map_err(Error::Term)?,
        args,
        config.clone(),
        true,
    )?;

    if let Some(keys_string) = &args.keys {
        let ("", keys) = key_parser::parse_keys(keys_string).expect("Couldn't parse keys") else {
            panic!("Couldn't parse keys");
        };

        for event in keys_to_events(&keys) {
            state.handle_event(term, event)?;
        }
    }

    state.redraw_now(term)?;

    if args.print {
        return Ok(());
    }

    state.run(term)?;

    Ok(())
}

fn poll_term_event() -> Option<Res<GituEvent>> {
    let is_event_found = match event::poll(Duration::from_millis(100)) {
        Ok(found) => found,
        Err(error) => return Some(Err(Error::Term(error))),
    };

    if is_event_found {
        Some(event::read().map(GituEvent::Term).map_err(Error::Term))
    } else {
        None
    }
}

fn poll_file_watcher(watcher: Option<Rc<FileWatcher>>) -> Option<Res<GituEvent>> {
    watcher
        .as_ref()
        .is_some_and(|w| w.pending_updates())
        .then(|| Ok(GituEvent::FileUpdate))
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

fn keys_to_events(keys: &[(KeyModifiers, KeyCode)]) -> Vec<GituEvent> {
    keys.iter()
        .map(|(mods, key)| {
            GituEvent::Term(Event::Key(KeyEvent {
                code: *key,
                modifiers: *mods,
                kind: event::KeyEventKind::Press,
                state: KeyEventState::NONE,
            }))
        })
        .collect::<Vec<_>>()
}
