pub mod cli;
mod git;
mod git2_opts;
mod items;
mod keybinds;
mod screen;
mod theme;
mod ui;

use crossterm::event::{self, Event, KeyEventKind};
use git2::Repository;
use items::{Item, TargetData};
use itertools::Itertools;
use keybinds::{Op, SubmenuOp, TargetOp};
use ratatui::{prelude::*, Terminal};
use screen::Screen;
use std::{
    borrow::Cow,
    error::Error,
    iter,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
};
use strum::IntoEnumIterator;
use tui_prompts::{prelude::*, State as _};

type Res<T> = Result<T, Box<dyn Error>>;

pub(crate) struct CmdMeta {
    pub(crate) args: Cow<'static, str>,
    pub(crate) out: Option<String>,
}

pub struct State {
    pub repo: Rc<Repository>,
    quit: bool,
    screens: Vec<Screen>,
    pending_submenu_op: SubmenuOp,
    pub(crate) cmd_meta: Option<CmdMeta>,
    prompt: Option<Op>,
    prompt_state: TextState<'static>,
}

impl State {
    pub fn create(repo: Repository, size: Rect, args: cli::Args) -> Res<Self> {
        let repo = Rc::new(repo);

        let screens = match args.command {
            Some(cli::Commands::Show { reference }) => {
                vec![screen::show::create(Rc::clone(&repo), size, reference)?]
            }
            None => vec![screen::status::create(Rc::clone(&repo), size)?],
        };

        Ok(Self {
            repo,
            quit: args.exit_immediately,
            screens,
            pending_submenu_op: SubmenuOp::None,
            cmd_meta: None,
            prompt: None,
            prompt_state: TextState::new(),
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
        self.cmd_meta = Some(CmdMeta {
            args: display.into(),
            out: None,
        });
        terminal.draw(|frame| ui::ui::<B>(frame, self))?;

        self.cmd_meta.as_mut().unwrap().out = Some(cmd(self)?);
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

        self.cmd_meta = Some(CmdMeta {
            args: command_args(&cmd),
            out: Some(
                String::from_utf8(out.stderr.clone())
                    .expect("Error turning command output to String"),
            ),
        });

        terminal.clear()?;
        self.screen_mut().update()?;

        Ok(())
    }
}

fn command_args(cmd: &Command) -> Cow<'static, str> {
    iter::once(cmd.get_program().to_string_lossy())
        .chain(cmd.get_args().map(|arg| arg.to_string_lossy()))
        .join(" ")
        .into()
}

pub fn run<B: Backend>(args: cli::Args, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    let dir = PathBuf::from(
        String::from_utf8(
            Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()?
                .stdout,
        )?
        .trim_end(),
    );

    let repo = Repository::open_from_env()?;
    repo.set_workdir(&dir, false)?;

    let mut state = State::create(repo, terminal.size()?, args)?;
    terminal.draw(|frame| ui::ui::<B>(frame, &mut state))?;

    while !state.quit {
        // TODO Gather all events, no need to draw for every
        if !event::poll(std::time::Duration::MAX)? {
            continue;
        }

        let event = event::read()?;
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
                if state.prompt_state.is_focused() {
                    state.prompt_state.handle_key_event(key)
                } else if key.kind == KeyEventKind::Press {
                    state.cmd_meta = None;

                    handle_op(terminal, state, key)?;
                }
            }
            _ => (),
        }

        match state.prompt_state.status() {
            Status::Pending => (),
            Status::Aborted => {
                state.prompt = None;
                state.prompt_state = TextState::new();
            }
            Status::Done => {
                if state.prompt == Some(Op::CheckoutNewBranch) {
                    let name = state.prompt_state.value().to_string();
                    let mut fun = cmd_arg(git::checkout_new_branch_cmd, &name).unwrap();
                    fun(terminal, state)?;
                }

                state.prompt = None;
                state.prompt_state = TextState::new();
            }
        }
    }

    if let Some(screen) = state.screens.last_mut() {
        screen.clamp_cursor();
        terminal.draw(|frame| ui::ui::<B>(frame, state))?;
    }

    Ok(())
}

fn handle_op<B: Backend>(
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
        use Op::*;
        let was_submenu = state.pending_submenu_op != SubmenuOp::None;
        state.pending_submenu_op = SubmenuOp::None;

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
                state.prompt = Some(Op::CheckoutNewBranch);
                state.prompt_state.focus();
            }
            Commit => {
                state.issue_subscreen_command(terminal, git::commit_cmd())?;
            }
            CommitAmend => {
                state.issue_subscreen_command(terminal, git::commit_amend_cmd())?;
            }
            Submenu(op) => state.pending_submenu_op = op,
            LogCurrent => goto_log_screen(Rc::clone(&state.repo), &mut state.screens, None),
            FetchAll => state.run_external_cmd(terminal, &[], git::fetch_all_cmd())?,
            Pull => state.run_external_cmd(terminal, &[], git::pull_cmd())?,
            Push => state.run_external_cmd(terminal, &[], git::push_cmd())?,
            Target(target_op) => {
                if let Some(act) = &state.screen_mut().get_selected_item().target_data.clone() {
                    if let Some(mut closure) = closure_by_target_op(act, &target_op) {
                        closure(terminal, state)?;
                    }
                }
            }
            RebaseAbort => {
                state.run_external_cmd(terminal, &[], git::rebase_abort_cmd())?;
            }
            RebaseContinue => {
                state.run_external_cmd(terminal, &[], git::rebase_continue_cmd())?;
            }
            ShowRefs => goto_refs_screen(Rc::clone(&state.repo), &mut state.screens),
        }
    }

    Ok(())
}

pub(crate) fn list_target_ops<B: Backend>(
    target: &TargetData,
) -> impl Iterator<Item = TargetOp> + '_ {
    TargetOp::iter().filter(|target_op| closure_by_target_op::<B>(target, target_op).is_some())
}

type OpClosure<'a, B> = Box<dyn FnMut(&mut Terminal<B>, &mut State) -> Res<()> + 'a>;

/// Retrieves the 'implementation' of a `TargetOp`.
/// These are `Option<OpClosure>`s, so that the mappings
/// can be introspected.
pub(crate) fn closure_by_target_op<'a, B: Backend>(
    target: &'a TargetData,
    target_op: &TargetOp,
) -> Option<OpClosure<'a, B>> {
    use TargetData::*;
    use TargetOp::*;

    match (target_op, target) {
        (Show, Commit(r) | Branch(r)) => goto_show_screen(r.clone()),
        (Show, File(u)) => editor(u.clone(), None),
        (Show, Delta(d)) => editor(d.new_file.clone(), None),
        (Show, Hunk(h)) => editor(h.new_file.clone(), Some(h.first_diff_line())),
        (Stage, File(u)) => cmd_arg(git::stage_file_cmd, u),
        (Stage, Delta(d)) => cmd_arg(git::stage_file_cmd, &d.new_file),
        (Stage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::stage_patch_cmd),
        (Unstage, Delta(d)) => cmd_arg(git::unstage_file_cmd, &d.new_file),
        (Unstage, Hunk(h)) => cmd(h.format_patch().into_bytes(), git::unstage_patch_cmd),
        (RebaseInteractive, Commit(r) | Branch(r)) => subscreen_arg(git::rebase_interactive_cmd, r),
        (CommitFixup, Commit(r)) => subscreen_arg(git::commit_fixup_cmd, r),
        (RebaseAutosquash, Commit(r) | Branch(r)) => subscreen_arg(git::rebase_autosquash_cmd, r),
        (ResetSoft, Commit(r) | Branch(r)) => cmd_arg(git::reset_soft_cmd, r),
        (ResetMixed, Commit(r) | Branch(r)) => cmd_arg(git::reset_mixed_cmd, r),
        (ResetHard, Commit(r) | Branch(r)) => cmd_arg(git::reset_hard_cmd, r),
        (Discard, Branch(r)) => cmd_arg(git::discard_branch, r),
        (Discard, File(f)) => Some(Box::new(|_term, state| {
            let path = PathBuf::from_iter([
                state.repo.workdir().expect("No workdir").to_path_buf(),
                f.into(),
            ]);
            std::fs::remove_file(path)?;
            state.screen_mut().update()
        })),
        (Discard, Delta(d)) => {
            if d.old_file == d.new_file {
                cmd_arg(git::checkout_file_cmd, &d.old_file)
            } else {
                // TODO Discard file move
                None
            }
        }
        (Discard, Hunk(h)) => cmd(
            h.format_patch().into_bytes(),
            git::discard_unstaged_patch_cmd,
        ),
        (Checkout, Commit(r) | Branch(r)) => cmd_arg(git::checkout_ref_cmd, r),
        (LogOther, Commit(r) | Branch(r)) => Some(Box::new(|_term, state| {
            goto_log_screen(Rc::clone(&state.repo), &mut state.screens, Some(r.clone()));
            Ok(())
        })),
        (_, _) => None,
    }
}

fn goto_show_screen<'a, B: Backend>(r: String) -> Option<OpClosure<'a, B>> {
    Some(Box::new(move |terminal, state| {
        state.screens.push(
            screen::show::create(Rc::clone(&state.repo), terminal.size()?, r.clone())
                .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}

fn editor<'a, B: Backend>(file: String, line: Option<u32>) -> Option<OpClosure<'a, B>> {
    Some(Box::new(move |terminal, state| {
        let file: &str = &file;
        let editor = std::env::var("EDITOR").expect("EDITOR not set");
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
            .expect("Error opening editor");

        state.screen_mut().update()
    }))
}

fn cmd<'a, B: Backend>(input: Vec<u8>, command: fn() -> Command) -> Option<OpClosure<'a, B>> {
    Some(Box::new(move |terminal, state| {
        state.run_external_cmd(terminal, &input, command())
    }))
}

fn cmd_arg<B: Backend>(command: fn(&str) -> Command, arg: &str) -> Option<OpClosure<B>> {
    Some(Box::new(move |terminal, state| {
        state.run_external_cmd(terminal, &[], command(arg))
    }))
}

fn subscreen_arg<B: Backend>(command: fn(&str) -> Command, arg: &str) -> Option<OpClosure<B>> {
    Some(Box::new(move |terminal, state| {
        state.issue_subscreen_command(terminal, command(arg))
    }))
}

fn goto_log_screen(repo: Rc<Repository>, screens: &mut Vec<Screen>, reference: Option<String>) {
    screens.drain(1..);
    let size = screens.last().unwrap().size;
    screens.push(screen::log::create(repo, size, reference).expect("Couldn't create screen"));
}

fn goto_refs_screen(repo: Rc<Repository>, screens: &mut Vec<Screen>) {
    screens.drain(1..);
    let size = screens.last().unwrap().size;
    screens.push(screen::show_refs::create(repo, size).expect("Couldn't create screen"));
}
