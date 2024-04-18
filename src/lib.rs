mod bindings;
pub mod cli;
mod cmd_log;
pub mod config;
mod git;
mod git2_opts;
mod items;
mod key_parser;
mod menu;
mod ops;
mod prompt;
mod screen;
pub mod state;
mod syntax_highlight;
pub mod term;
#[cfg(test)]
mod tests;
mod ui;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers};
use git2::Repository;
use items::Item;
use ops::Action;
use std::{error::Error, path::PathBuf, process::Command, rc::Rc, time::Duration};
use term::Term;

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

pub type Res<T> = Result<T, Box<dyn Error>>;

pub fn run(args: &cli::Args, term: &mut Term) -> Res<()> {
    log::debug!("Finding git dir");
    let dir = PathBuf::from(
        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()?
                .stdout,
        )?
        .trim_end(),
    );

    log::debug!("Opening repo");
    let repo = Repository::open_from_env()?;
    repo.set_workdir(&dir, false)?;

    log::debug!("Initializing config");
    let config = config::init_config()?;

    log::debug!("Creating initial state");
    let mut state = state::State::create(Rc::new(repo), term.size()?, args, Rc::new(config), true)?;

    log::debug!("Initial update");
    state.update(term, &[Event::FocusGained])?;

    if args.print {
        return Ok(());
    }

    if let Some(keys_string) = &args.keys {
        let ("", keys) = key_parser::parse_keys(keys_string).expect("Couldn't parse keys") else {
            panic!("Couldn't parse keys");
        };
        handle_initial_send_keys(&keys, &mut state, term)?;
    }

    while !state.quit {
        let events = if event::poll(Duration::from_millis(100))? {
            vec![event::read()?]
        } else {
            vec![]
        };

        state.update(term, &events)?;
    }

    Ok(())
}

fn handle_initial_send_keys(
    keys: &[(KeyModifiers, KeyCode)],
    state: &mut state::State,
    term: &mut ratatui::prelude::Terminal<term::TermBackend>,
) -> Result<(), Box<dyn Error>> {
    let initial_events = keys
        .iter()
        .map(|(mods, key)| {
            Event::Key(KeyEvent {
                code: *key,
                modifiers: *mods,
                kind: event::KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
        })
        .collect::<Vec<_>>();

    if !initial_events.is_empty() {
        state.update(term, &initial_events)?;
    }

    Ok(())
}
