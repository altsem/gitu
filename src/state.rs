use std::borrow::Cow;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEventKind;
use git2::Repository;
use ratatui::layout::Rect;
use tui_prompts::State as _;
use tui_prompts::Status;

use crate::cli;
use crate::config::Config;
use crate::handle_op;
use crate::keybinds;
use crate::menu::Menu;
use crate::menu::PendingMenu;
use crate::prompt;
use crate::screen;
use crate::screen::Screen;
use crate::term;
use crate::term::Term;
use crate::ui;

use super::command_args;
use super::CmdMetaBuffer;
use super::ErrorBuffer;
use super::Res;

pub struct State {
    pub repo: Rc<Repository>,
    pub(crate) config: Rc<Config>,
    pub quit: bool,
    pub(crate) screens: Vec<Screen>,
    pub(crate) pending_menu: Option<PendingMenu>,
    pub(crate) cmd_meta_buffer: Option<CmdMetaBuffer>,
    pub(crate) error_buffer: Option<ErrorBuffer>,
    pub(crate) prompt: prompt::Prompt,
    next_input_is_arg: bool,
}

impl State {
    pub fn create(repo: Repository, size: Rect, args: &cli::Args, config: Config) -> Res<Self> {
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
            pending_menu: None,
            cmd_meta_buffer: None,
            error_buffer: None,
            prompt: prompt::Prompt::new(),
            next_input_is_arg: false,
        })
    }

    pub fn update(&mut self, term: &mut Term, events: &[Event]) -> Res<()> {
        for event in events {
            match *event {
                Event::Resize(w, h) => {
                    for screen in self.screens.iter_mut() {
                        screen.size = Rect::new(0, 0, w, h);
                    }
                }
                Event::Key(key) => {
                    if self.prompt.state.is_focused() {
                        self.prompt.state.handle_key_event(key)
                    } else if key.kind == KeyEventKind::Press {
                        self.cmd_meta_buffer = None;
                        self.error_buffer = None;

                        self.handle_key_input(term, key)?;
                    }
                }
                _ => (),
            }

            self.update_prompt(term)?;
        }

        if self.screens.last_mut().is_some() {
            term.draw(|frame| ui::ui(frame, self))?;
        }

        Ok(())
    }

    pub(crate) fn update_prompt(&mut self, term: &mut Term) -> Res<()> {
        if self.prompt.state.status() == Status::Aborted {
            self.prompt.reset(term)?;
        } else if let Some(mut prompt_data) = self.prompt.data.take() {
            (Rc::get_mut(&mut prompt_data.update_fn).unwrap())(self, term)?;
            if self.prompt.state.is_focused() {
                self.prompt.data = Some(prompt_data);
            }
        }

        Ok(())
    }

    pub(crate) fn handle_key_input(&mut self, term: &mut Term, key: event::KeyEvent) -> Res<()> {
        let pending = match &self.pending_menu {
            None => None,
            Some(menu) if menu.menu == Menu::Help => None,
            Some(menu) => Some(menu.menu),
        };

        let maybe_op = if self.next_input_is_arg {
            keybinds::arg_op_of_key_event(pending, key)
        } else {
            keybinds::op_of_key_event(pending, key)
        };

        self.next_input_is_arg = pending.is_some() && key.code == KeyCode::Char('-');

        if let Some(op) = maybe_op {
            let result = handle_op(self, op, term);

            if let Err(error) = result {
                self.error_buffer = Some(ErrorBuffer(error.to_string()));
            }
        }

        Ok(())
    }

    pub(crate) fn screen_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("No screen")
    }

    pub(crate) fn screen(&self) -> &Screen {
        self.screens.last().expect("No screen")
    }

    pub(crate) fn run_external_cmd(
        &mut self,
        term: &mut Term,
        input: &[u8],
        mut cmd: Command,
    ) -> Res<()> {
        cmd.current_dir(self.repo.workdir().expect("No workdir"));

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        self.run_cmd(term, command_args(&cmd), |_state| {
            let mut child = cmd.spawn()?;

            use std::io::Write;
            child.stdin.take().unwrap().write_all(input)?;

            let out = String::from_utf8(child.wait_with_output()?.stderr.clone())
                .expect("Error turning command output to String");

            Ok(out)
        })?;

        Ok(())
    }

    pub(crate) fn run_cmd<S: Into<Cow<'static, str>>, F: FnMut(&mut Self) -> Res<String>>(
        &mut self,
        term: &mut Term,
        display: S,
        mut cmd: F,
    ) -> Res<()> {
        self.cmd_meta_buffer = Some(CmdMetaBuffer {
            args: display.into(),
            out: None,
        });
        term.draw(|frame| ui::ui(frame, self))?;

        self.cmd_meta_buffer.as_mut().unwrap().out = Some(cmd(self)?);
        self.screen_mut().update()?;

        Ok(())
    }

    pub(crate) fn issue_subscreen_command(&mut self, term: &mut Term, mut cmd: Command) -> Res<()> {
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
        term.hide_cursor()?;

        // In case the command left the alternate screen (editors would)
        term::enter_alternate_screen()?;

        term.clear()?;
        self.screen_mut().update()?;

        Ok(())
    }
}
