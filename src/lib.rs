pub mod cli;
pub mod config;
mod git;
mod git2_opts;
mod items;
mod keybinds;
mod prompt;
mod screen;
pub mod term;
mod ui;

use config::Config;
use crossterm::event::{self, Event, KeyEventKind};
use git2::Repository;
use items::{Item, TargetData};
use itertools::Itertools;
use keybinds::{Op, SubmenuOp, TargetOp};
use ratatui::prelude::*;
use screen::Screen;
use std::{
    borrow::Cow,
    error::Error,
    ffi::{OsStr, OsString},
    iter,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
};
use strum::IntoEnumIterator;
use tui_prompts::{prelude::*, State as _};

const APP_NAME: &str = "gitu";

pub type Res<T> = Result<T, Box<dyn Error>>;

pub(crate) struct CmdMetaBuffer {
    pub(crate) args: Cow<'static, str>,
    pub(crate) out: Option<String>,
}

pub(crate) struct ErrorBuffer(String);

pub struct State {
    pub repo: Rc<Repository>,
    config: Rc<Config>,
    quit: bool,
    screens: Vec<Screen>,
    pending_submenu_op: SubmenuOp,
    pub(crate) cmd_meta_buffer: Option<CmdMetaBuffer>,
    pub(crate) error_buffer: Option<ErrorBuffer>,
    prompt: prompt::Prompt,
}

impl State {
    pub fn create(
        repo: Repository,
        size: Rect,
        args: &cli::Args,
        config: config::Config,
    ) -> Res<Self> {
        let repo = Rc::new(repo);
        let config = Rc::new(config);

        let screens = match args.command {
            Some(cli::Commands::Show { ref reference }) => {
                vec![screen::show::create(
                    Rc::clone(&config),
                    Rc::clone(&repo),
                    size,
                    reference.clone(),
                )?]
            }
            None => vec![screen::status::create(
                Rc::clone(&config),
                Rc::clone(&repo),
                size,
            )?],
        };

        Ok(Self {
            repo,
            config,
            quit: false,
            screens,
            pending_submenu_op: SubmenuOp::None,
            cmd_meta_buffer: None,
            error_buffer: None,
            prompt: prompt::Prompt::new(),
        })
    }

    fn screen_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("No screen")
    }

    fn screen(&self) -> &Screen {
        self.screens.last().expect("No screen")
    }

    pub(crate) fn run_external_cmd<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        input: &[u8],
        mut cmd: Command,
    ) -> Res<()> {
        cmd.current_dir(self.repo.workdir().expect("No workdir"));

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        self.run_cmd(terminal, command_args(&cmd), |_state| {
            let mut child = cmd.spawn()?;

            use std::io::Write;
            child.stdin.take().unwrap().write_all(input)?;

            let out = String::from_utf8(child.wait_with_output()?.stderr.clone())
                .expect("Error turning command output to String");

            Ok(out)
        })?;

        Ok(())
    }

    pub(crate) fn run_cmd<
        B: Backend,
        S: Into<Cow<'static, str>>,
        F: FnMut(&mut Self) -> Res<String>,
    >(
        &mut self,
        terminal: &mut Terminal<B>,
        display: S,
        mut cmd: F,
    ) -> Res<()> {
        self.cmd_meta_buffer = Some(CmdMetaBuffer {
            args: display.into(),
            out: None,
        });
        terminal.draw(|frame| ui::ui::<B>(frame, self))?;

        self.cmd_meta_buffer.as_mut().unwrap().out = Some(cmd(self)?);
        self.screen_mut().update()?;

        Ok(())
    }

    pub(crate) fn issue_subscreen_command<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        mut cmd: Command,
    ) -> Res<()> {
        cmd.current_dir(self.repo.workdir().expect("No workdir"));

        cmd.stdin(Stdio::piped());
        let child = cmd.spawn()?;

        let out = child.wait_with_output()?;

        self.cmd_meta_buffer = Some(CmdMetaBuffer {
            args: command_args(&cmd),
            out: Some(
                String::from_utf8(out.stderr.clone())
                    .expect("Error turning command output to String"),
            ),
        });

        // Prevents cursor flash when exiting editor
        terminal.hide_cursor()?;

        // In case the command left the alternate screen (editors would)
        term::enter_alternate_screen()?;

        terminal.clear()?;
        self.screen_mut().update()?;

        Ok(())
    }

    fn goto_log_screen(&mut self, reference: Option<String>) {
        self.screens.drain(1..);
        let size = self.screens.last().unwrap().size;
        self.screens.push(
            screen::log::create(
                Rc::clone(&self.config),
                Rc::clone(&self.repo),
                size,
                reference,
            )
            .expect("Couldn't create screen"),
        );
    }

    fn goto_refs_screen(&mut self) {
        self.screens.drain(1..);
        let size = self.screens.last().unwrap().size;
        self.screens.push(
            screen::show_refs::create(Rc::clone(&self.config), Rc::clone(&self.repo), size)
                .expect("Couldn't create screen"),
        );
    }
}

fn command_args(cmd: &Command) -> Cow<'static, str> {
    iter::once(cmd.get_program().to_string_lossy())
        .chain(cmd.get_args().map(|arg| arg.to_string_lossy()))
        .join(" ")
        .into()
}

pub fn run<B: Backend>(args: &cli::Args, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
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
    let mut state = State::create(repo, terminal.size()?, args, config)?;

    log::debug!("Drawing initial frame");
    terminal.draw(|frame| ui::ui::<B>(frame, &mut state))?;

    if args.print {
        return Ok(());
    }

    while !state.quit {
        log::debug!("Awaiting event");
        let event = event::read()?;

        log::debug!("Updating");
        update(terminal, &mut state, &[event])?;
    }

    Ok(())
}

pub fn update<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut State,
    events: &[Event],
) -> Res<()> {
    for event in events {
        match *event {
            Event::Resize(w, h) => {
                for screen in state.screens.iter_mut() {
                    screen.size = Rect::new(0, 0, w, h);
                }
            }
            Event::Key(key) => {
                if state.prompt.state.is_focused() {
                    state.prompt.state.handle_key_event(key)
                } else if key.kind == KeyEventKind::Press {
                    state.cmd_meta_buffer = None;
                    state.error_buffer = None;

                    handle_key_input(terminal, state, key)?;
                }
            }
            _ => (),
        }

        update_prompt(state, terminal)?;
    }

    if state.screens.last_mut().is_some() {
        terminal.draw(|frame| ui::ui::<B>(frame, state))?;
    }

    Ok(())
}

fn update_prompt<B: Backend>(state: &mut State, terminal: &mut Terminal<B>) -> Res<()> {
    if state.prompt.state.status() == Status::Aborted {
        state.prompt.reset(terminal)?;
    } else if let Some(pending_prompt) = state.prompt.pending_op {
        match (state.prompt.state.status(), pending_prompt) {
            (Status::Done, Op::CheckoutNewBranch) => {
                let name = state.prompt.state.value().to_string();
                cmd_arg(git::checkout_new_branch_cmd, name.into()).unwrap()(terminal, state)?;
                state.prompt.reset(terminal)?;
            }
            (Status::Pending, Op::Target(TargetOp::Discard)) => match state.prompt.state.value() {
                "y" => {
                    let mut action =
                        get_action(clone_target_data(state), TargetOp::Discard).unwrap();
                    action(terminal, state)?;
                    state.prompt.reset(terminal)?;
                }
                "" => (),
                _ => {
                    state.error_buffer = Some(ErrorBuffer(format!("{:?} aborted", pending_prompt)));
                    state.prompt.reset(terminal)?;
                }
            },
            _ => (),
        }
    }

    Ok(())
}

fn clone_target_data(state: &mut State) -> Option<TargetData> {
    let screen = state.screen();
    let selected = screen.get_selected_item();
    selected.target_data.clone()
}

fn handle_key_input<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &mut State,
    key: event::KeyEvent,
) -> Res<()> {
    let pending = if state.pending_submenu_op == SubmenuOp::Help {
        SubmenuOp::None
    } else {
        state.pending_submenu_op
    };

    if let Some(op) = keybinds::op_of_key_event(pending, key) {
        let was_submenu = state.pending_submenu_op != SubmenuOp::None;
        state.pending_submenu_op = SubmenuOp::None;

        let result = handle_op(op, was_submenu, state, terminal);

        if let Err(error) = result {
            state.error_buffer = Some(ErrorBuffer(error.to_string()));
        }
    }

    Ok(())
}

fn handle_op<B: Backend>(
    op: Op,
    was_submenu: bool,
    state: &mut State,
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn Error>> {
    use Op::*;

    match op {
        Quit => {
            if was_submenu {
                // Do nothing, already cleared
            } else {
                state.screens.pop();
                if let Some(screen) = state.screens.last_mut() {
                    screen.update()?;
                } else {
                    state.quit = true
                }
            }
        }
        Refresh => state.screen_mut().update()?,
        ToggleSection => state.screen_mut().toggle_section(),
        SelectPrevious => state.screen_mut().select_previous(),
        SelectNext => state.screen_mut().select_next(),
        HalfPageUp => state.screen_mut().scroll_half_page_up(),
        HalfPageDown => state.screen_mut().scroll_half_page_down(),
        CheckoutNewBranch => {
            state.prompt.set(Op::CheckoutNewBranch);
        }
        Commit => {
            state.issue_subscreen_command(terminal, git::commit_cmd())?;
        }
        CommitAmend => {
            state.issue_subscreen_command(terminal, git::commit_amend_cmd())?;
        }
        Submenu(op) => state.pending_submenu_op = op,
        LogCurrent => state.goto_log_screen(None),
        FetchAll => state.run_external_cmd(terminal, &[], git::fetch_all_cmd())?,
        Pull => state.run_external_cmd(terminal, &[], git::pull_cmd())?,
        Push => state.run_external_cmd(terminal, &[], git::push_cmd())?,
        Target(TargetOp::Discard) => prompt_action::<B>(state, Target(TargetOp::Discard)),
        Target(target_op) => {
            if let Some(mut action) = get_action(clone_target_data(state), target_op) {
                action(terminal, state)?
            }
        }
        RebaseAbort => {
            state.run_external_cmd(terminal, &[], git::rebase_abort_cmd())?;
        }
        RebaseContinue => {
            state.run_external_cmd(terminal, &[], git::rebase_continue_cmd())?;
        }
        ShowRefs => state.goto_refs_screen(),
    }

    Ok(())
}

fn get_action<B: Backend>(
    target_data: Option<TargetData>,
    target_op: TargetOp,
) -> Option<Action<B>> {
    target_data.and_then(|data| action_by_target_op::<B>(data, &target_op))
}

fn prompt_action<B: Backend>(state: &mut State, op: Op) {
    if let Op::Target(target_op) = op {
        if get_action::<B>(clone_target_data(state), target_op).is_none() {
            return;
        }
    }

    state.prompt.set(op);
}

pub(crate) fn list_target_ops<B: Backend>(
    data: &TargetData,
) -> impl Iterator<Item = (TargetOp, TargetData)> + '_ {
    TargetOp::iter()
        .filter(|target_op| action_by_target_op::<B>(data.clone(), target_op).is_some())
        .map(|op| (op, data.clone()))
}

type Action<B> = Box<dyn FnMut(&mut Terminal<B>, &mut State) -> Res<()>>;

/// Retrieves the 'implementation' of a `TargetOp`.
/// These are `Option<OpClosure>`s, so that the mappings
/// can be introspected.
pub(crate) fn action_by_target_op<B: Backend>(
    target: TargetData,
    target_op: &TargetOp,
) -> Option<Action<B>> {
    use TargetData::*;
    use TargetOp::*;

    match (target_op, target) {
        (Show, Commit(r) | Branch(r)) => goto_show_screen(r.clone()),
        (Show, File(u)) => editor(u.as_path(), None),
        (Show, Delta(d)) => editor(d.new_file.as_path(), None),
        (Show, Hunk(h)) => editor(h.new_file.as_path(), Some(h.first_diff_line())),
        (Stage, File(u)) => cmd_arg(git::stage_file_cmd, u.into()),
        (Stage, Delta(d)) => cmd_arg(git::stage_file_cmd, d.new_file.into()),
        (Stage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
        (Unstage, Delta(d)) => cmd_arg(git::unstage_file_cmd, d.new_file.into()),
        (Unstage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::unstage_patch_cmd),
        (RebaseInteractive, Commit(r) | Branch(r)) => {
            subscreen_arg(git::rebase_interactive_cmd, r.into())
        }
        (CommitFixup, Commit(r)) => subscreen_arg(git::commit_fixup_cmd, r.into()),
        (RebaseAutosquash, Commit(r) | Branch(r)) => {
            subscreen_arg(git::rebase_autosquash_cmd, r.into())
        }
        (ResetSoft, Commit(r) | Branch(r)) => cmd_arg(git::reset_soft_cmd, r.into()),
        (ResetMixed, Commit(r) | Branch(r)) => cmd_arg(git::reset_mixed_cmd, r.into()),
        (ResetHard, Commit(r) | Branch(r)) => cmd_arg(git::reset_hard_cmd, r.into()),
        (Discard, Branch(r)) => cmd_arg(git::discard_branch, r.into()),
        (Discard, File(f)) => Some(Box::new(move |_term, state| {
            let path = PathBuf::from_iter([
                state.repo.workdir().expect("No workdir").to_path_buf(),
                f.clone(),
            ]);
            std::fs::remove_file(path)?;
            state.screen_mut().update()
        })),
        (Discard, Delta(d)) => {
            if d.old_file == d.new_file {
                cmd_arg(git::checkout_file_cmd, d.old_file.into())
            } else {
                // TODO Discard file move
                None
            }
        }
        (Discard, Hunk(h)) => cmd(
            h.format_patch().into_bytes(),
            git::discard_unstaged_patch_cmd,
        ),
        (Checkout, Commit(r) | Branch(r)) => cmd_arg(git::checkout_ref_cmd, r.into()),
        (LogOther, Commit(r) | Branch(r)) => Some(Box::new(move |_term, state| {
            state.goto_log_screen(Some(r.clone()));
            Ok(())
        })),
        (_, _) => None,
    }
}

fn goto_show_screen<B: Backend>(r: String) -> Option<Action<B>> {
    Some(Box::new(move |terminal, state| {
        state.screens.push(
            screen::show::create(
                Rc::clone(&state.config),
                Rc::clone(&state.repo),
                terminal.size()?,
                r.clone(),
            )
            .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}

fn editor<B: Backend>(file: &Path, line: Option<u32>) -> Option<Action<B>> {
    let file = file.to_str().unwrap().to_string();

    Some(Box::new(move |terminal, state| {
        const EDITOR_VARS: [&str; 3] = ["GIT_EDITOR", "VISUAL", "EDITOR"];
        let configured_editor = EDITOR_VARS
            .into_iter()
            .find_map(|var| std::env::var(var).ok());

        let Some(editor) = configured_editor else {
            return Err(format!(
                "No editor environment variable set ({})",
                EDITOR_VARS.join(", ")
            )
            .into());
        };

        let mut cmd = Command::new(editor.clone());
        let args = match line {
            Some(line) => match editor.as_str() {
                "vi" | "vim" | "nvim" | "nano" => {
                    vec![format!("+{}", line), file.to_string()]
                }
                _ => vec![format!("{}:{}", file, line)],
            },
            None => vec![file.to_string()],
        };

        cmd.args(args);

        state
            .issue_subscreen_command(terminal, cmd)
            .map_err(|err| format!("Couldn't open editor {} due to: {}", editor, err))?;

        state.screen_mut().update()
    }))
}

fn cmd<B: Backend>(input: Vec<u8>, command: fn() -> Command) -> Option<Action<B>> {
    Some(Box::new(move |terminal, state| {
        state.run_external_cmd(terminal, &input, command())
    }))
}

fn cmd_arg<B: Backend>(command: fn(&OsStr) -> Command, arg: OsString) -> Option<Action<B>> {
    Some(Box::new(move |terminal, state| {
        state.run_external_cmd(terminal, &[], command(&arg))
    }))
}

fn subscreen_arg<B: Backend>(command: fn(&OsStr) -> Command, arg: OsString) -> Option<Action<B>> {
    Some(Box::new(move |terminal, state| {
        state.issue_subscreen_command(terminal, command(&arg))
    }))
}
