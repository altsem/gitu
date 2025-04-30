use std::io::Read;
use std::io::Write;
use std::ops::DerefMut;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

use arboard::Clipboard;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use git2::Repository;
use ratatui::layout::Size;
use tui_prompts::State as _;

use crate::bindings::Bindings;
use crate::cli;
use crate::cmd_log::CmdLog;
use crate::cmd_log::CmdLogEntry;
use crate::config::Config;
use crate::error::Error;
use crate::file_watcher::FileWatcher;
use crate::items::TargetData;
use crate::menu::Menu;
use crate::menu::PendingMenu;
use crate::ops::Op;
use crate::prompt;
use crate::prompt::PromptData;
use crate::screen;
use crate::screen::Screen;
use crate::term::Term;
use crate::ui;

use super::Res;

pub(crate) struct State {
    pub repo: Rc<Repository>,
    pub config: Rc<Config>,
    pub bindings: Bindings,
    pending_keys: Vec<(KeyModifiers, KeyCode)>,
    pub quit: bool,
    pub screens: Vec<Screen>,
    pub pending_menu: Option<PendingMenu>,
    pending_cmd: Option<(Child, Arc<RwLock<CmdLogEntry>>)>,
    enable_async_cmds: bool,
    pub current_cmd_log: CmdLog,
    pub prompt: prompt::Prompt,
    pub clipboard: Option<Clipboard>,
    needs_redraw: bool,
    file_watcher: Option<FileWatcher>,
}

impl State {
    pub fn create(
        repo: Rc<Repository>,
        size: Size,
        args: &cli::Args,
        config: Rc<Config>,
        enable_async_cmds: bool,
    ) -> Res<Self> {
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

        let bindings = Bindings::from(&config.bindings);
        let pending_menu = root_menu(&config).map(PendingMenu::init);

        let clipboard = Clipboard::new()
            .inspect_err(|e| log::warn!("Couldn't initialize clipboard: {}", e))
            .ok();

        let file_watcher = if config.general.refresh_on_file_change.enabled {
            Some(FileWatcher::new(
                repo.workdir().expect("Bare repos unhandled"),
            )?)
        } else {
            None
        };

        Ok(Self {
            repo,
            config,
            bindings,
            pending_keys: vec![],
            enable_async_cmds,
            quit: false,
            screens,
            pending_cmd: None,
            pending_menu,
            current_cmd_log: CmdLog::new(),
            prompt: prompt::Prompt::new(),
            clipboard,
            file_watcher,
            needs_redraw: true,
        })
    }

    pub fn run(&mut self, term: &mut Term, max_tick_delay: Duration) -> Res<()> {
        while !self.quit {
            term.backend_mut().poll_event(max_tick_delay)?;
            self.update(term)?;
        }

        Ok(())
    }

    pub fn update(&mut self, term: &mut Term) -> Res<()> {
        if term.backend_mut().poll_event(Duration::ZERO)? {
            let event = term.backend_mut().read_event()?;
            self.handle_event(term, event)?;
        }

        if let Some(file_watcher) = &mut self.file_watcher {
            if file_watcher.pending_updates() {
                self.screen_mut().update()?;
                self.stage_redraw();
            }
        }

        let handle_pending_cmd_result = self.handle_pending_cmd();
        self.handle_result(handle_pending_cmd_result)?;

        if self.needs_redraw {
            self.redraw_now(term)?;
        }

        Ok(())
    }

    pub fn handle_event(&mut self, term: &mut Term, event: Event) -> Res<()> {
        log::debug!("{:?}", event);

        match event {
            Event::Resize(w, h) => {
                for screen in self.screens.iter_mut() {
                    screen.size = Size::new(w, h);
                }

                self.stage_redraw();
                Ok(())
            }
            Event::Key(key) => {
                if self.pending_cmd.is_none() {
                    self.current_cmd_log.clear();
                }

                if self.prompt.state.is_focused() {
                    self.prompt.state.handle_key_event(key);
                } else {
                    self.handle_key_input(term, key)?;
                }

                self.stage_redraw();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn redraw_now(&mut self, term: &mut Term) -> Res<()> {
        if self.screens.last_mut().is_some() {
            term.draw(|frame| ui::ui(frame, self))
                .map_err(Error::Term)?;

            self.needs_redraw = false;
        };

        Ok(())
    }

    pub fn stage_redraw(&mut self) {
        self.needs_redraw = true;
    }

    fn handle_key_input(&mut self, term: &mut Term, key: event::KeyEvent) -> Res<()> {
        let menu = match &self.pending_menu {
            None => Menu::Root,
            Some(menu) if menu.menu == Menu::Help => Menu::Root,
            Some(menu) => menu.menu,
        };

        self.pending_keys.push((key.modifiers, key.code));
        let matching_bindings = self
            .bindings
            .match_bindings(&menu, &self.pending_keys)
            .collect::<Vec<_>>();

        match matching_bindings[..] {
            [binding] => {
                if binding.keys == self.pending_keys {
                    self.handle_op(binding.op.clone(), term)?;
                    self.pending_keys.clear();
                }
            }
            [] => self.pending_keys.clear(),
            [_, ..] => (),
        }

        Ok(())
    }

    pub(crate) fn handle_op(&mut self, op: Op, term: &mut Term) -> Res<()> {
        let target = self.screen().get_selected_item().target_data.as_ref();
        if let Some(mut action) = op.clone().implementation().get_action(target) {
            let result = Rc::get_mut(&mut action).unwrap()(self, term);
            self.handle_result(result)?;
        }

        Ok(())
    }

    fn handle_result<T>(&mut self, result: Res<T>) -> Res<()> {
        match result {
            Ok(_) => Ok(()),
            Err(Error::NoMoreEvents) => Err(Error::NoMoreEvents),
            Err(Error::PromptAborted) => Ok(()),
            Err(error) => {
                self.current_cmd_log
                    .push(CmdLogEntry::Error(error.to_string()));

                Ok(())
            }
        }
    }

    pub fn close_menu(&mut self) {
        self.pending_menu = root_menu(&self.config).map(PendingMenu::init)
    }

    pub fn screen_mut(&mut self) -> &mut Screen {
        self.screens.last_mut().expect("No screen")
    }

    pub fn screen(&self) -> &Screen {
        self.screens.last().expect("No screen")
    }

    /// Displays an `Info` message to the CmdLog.
    pub fn display_info(&mut self, message: String) {
        self.current_cmd_log.push(CmdLogEntry::Info(message));
    }

    /// Displays an `Error` message to the CmdLog.
    pub fn display_error(&mut self, error: String) {
        self.current_cmd_log.push(CmdLogEntry::Error(error));
    }

    /// Runs a `Command` and handles its output.
    /// Will block awaiting its completion.
    pub fn run_cmd(&mut self, term: &mut Term, input: &[u8], cmd: Command) -> Res<()> {
        self.run_cmd_async(term, input, cmd)?;
        self.await_pending_cmd()?;
        self.handle_pending_cmd()?;
        Ok(())
    }

    /// Runs a `Command` and handles its output asynchronously (if async commands are enabled).
    /// Will return `Ok(())` if one is already running.
    pub fn run_cmd_async(&mut self, term: &mut Term, input: &[u8], mut cmd: Command) -> Res<()> {
        if self.pending_cmd.is_some() {
            return Err(Error::CmdAlreadyRunning);
        }

        cmd.current_dir(self.repo.workdir().expect("No workdir"));

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let log_entry = self.current_cmd_log.push_cmd(&cmd);
        term.draw(|frame| ui::ui(frame, self))
            .map_err(Error::Term)?;

        let mut child = cmd.spawn().map_err(Error::SpawnCmd)?;

        use std::io::Write;
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input)
            .map_err(Error::Term)?;

        self.pending_cmd = Some((child, log_entry));

        if !self.enable_async_cmds {
            self.await_pending_cmd()?;
        }

        Ok(())
    }

    fn await_pending_cmd(&mut self) -> Res<()> {
        if let Some((child, _)) = &mut self.pending_cmd {
            child.wait().map_err(Error::CouldntAwaitCmd)?;
        }
        Ok(())
    }

    /// Handles any pending_cmd in State without blocking. Returns `true` if a cmd was handled.
    fn handle_pending_cmd(&mut self) -> Res<()> {
        let Some((ref mut child, ref mut log_rwlock)) = self.pending_cmd else {
            return Ok(());
        };

        let Some(status) = child.try_wait().map_err(Error::CouldntAwaitCmd)? else {
            return Ok(());
        };

        log::debug!("pending cmd finished with {:?}", status);

        let result = write_child_output_to_log(log_rwlock, child, status);
        self.pending_cmd = None;
        self.screen_mut().update()?;
        self.stage_redraw();
        result?;

        Ok(())
    }

    pub fn run_cmd_interactive(&mut self, term: &mut Term, mut cmd: Command) -> Res<()> {
        if self.pending_cmd.is_some() {
            return Err(Error::CmdAlreadyRunning);
        }

        cmd.current_dir(self.repo.workdir().ok_or(Error::NoRepoWorkdir)?);

        self.current_cmd_log.push_cmd_with_output(&cmd, "\n".into());
        self.redraw_now(term)?;

        eprint!("\r");

        // Redirect stderr so we can capture it via `Child::wait_with_output()`
        cmd.stderr(Stdio::piped());

        // git will have staircased output in raw mode (issue #290)
        // disable raw mode temporarily for the git command
        term.backend().disable_raw_mode()?;

        // If we don't show the cursor prior spawning (thus restore the default
        // state), the cursor may be missing in $EDITOR.
        term.show_cursor().map_err(Error::Term)?;

        let mut child = cmd.spawn().map_err(Error::SpawnCmd)?;

        // Drop stdin as `Child::wait_with_output` would
        drop(child.stdin.take());

        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

        tee(child.stdout.as_mut(), &mut [&mut stdout]).map_err(Error::Term)?;

        tee(
            child.stderr.as_mut(),
            &mut [&mut std::io::stderr(), &mut stderr],
        )
        .map_err(Error::Term)?;

        let status = child.wait().map_err(Error::CouldntAwaitCmd)?;
        let out_utf8 = String::from_utf8(strip_ansi_escapes::strip(stderr.clone()))
            .expect("Error turning command output to String")
            .into();

        self.current_cmd_log.clear();
        self.current_cmd_log.push_cmd_with_output(&cmd, out_utf8);

        // restore the raw mode
        term.backend().enable_raw_mode()?;

        // Prevents cursor flash when exiting editor
        term.hide_cursor().map_err(Error::Term)?;

        // In case the command left the alternate screen (editors would)
        term.backend_mut().enter_alternate_screen()?;

        term.clear().map_err(Error::Term)?;
        self.screen_mut().update()?;

        if !status.success() {
            return Err(Error::CmdBadExit(
                format!(
                    "{} {}",
                    cmd.get_program().to_string_lossy(),
                    cmd.get_args()
                        .map(|arg| arg.to_string_lossy())
                        .collect::<String>()
                ),
                status.code(),
            ));
        }

        Ok(())
    }

    pub fn hide_menu(&mut self) {
        if let Some(ref mut menu) = self.pending_menu {
            menu.is_hidden = true;
        }
    }

    pub fn unhide_menu(&mut self) {
        if let Some(ref mut menu) = self.pending_menu {
            menu.is_hidden = false;
        }
    }

    pub fn selected_rev(&self) -> Option<String> {
        match &self.screen().get_selected_item().target_data {
            Some(TargetData::Branch(branch)) => Some(branch.to_owned()),
            Some(TargetData::Commit(commit)) => Some(commit.to_owned()),
            _ => None,
        }
    }

    pub fn prompt(&mut self, term: &mut Term, params: &PromptParams) -> Res<String> {
        let prompt_text = if let Some(default) = (params.create_default_value)(self) {
            format!("{} (default {}):", params.prompt, default).into()
        } else {
            format!("{}:", params.prompt).into()
        };

        if params.hide_menu {
            self.hide_menu();
        }

        self.prompt.set(PromptData { prompt_text });
        self.redraw_now(term)?;

        loop {
            let event = term.backend_mut().read_event()?;
            self.handle_event(term, event)?;

            if self.prompt.state.status().is_done() {
                let value = get_prompt_result(params, self)?;

                self.unhide_menu();
                self.prompt.reset(term)?;

                return Ok(value);
            } else if self.prompt.state.status().is_aborted() {
                self.unhide_menu();
                self.prompt.reset(term)?;

                return Err(Error::PromptAborted);
            }

            self.redraw_now(term)?;
        }
    }

    pub fn confirm(&mut self, term: &mut Term, prompt: &'static str) -> Res<()> {
        self.hide_menu();
        self.prompt.set(PromptData {
            prompt_text: prompt.into(),
        });
        self.redraw_now(term)?;

        loop {
            let event = term.backend_mut().read_event()?;
            self.handle_event(term, event)?;

            match self.prompt.state.value() {
                "y" => {
                    self.prompt.reset(term)?;
                    return Ok(());
                }
                "" => (),
                _ => {
                    self.prompt.reset(term)?;
                    return Err(Error::PromptAborted);
                }
            }

            self.redraw_now(term)?;
        }
    }
}

fn get_prompt_result(params: &PromptParams, state: &mut State) -> Res<String> {
    let input = state.prompt.state.value();
    let default_value = (params.create_default_value)(state);

    let value = match (input, &default_value) {
        ("", None) => "",
        ("", Some(selected)) => selected,
        (value, _) => value,
    };

    Ok(value.to_string())
}

fn tee(maybe_input: Option<&mut impl Read>, outputs: &mut [&mut dyn Write]) -> std::io::Result<()> {
    let Some(input) = maybe_input else {
        return Ok(());
    };

    let mut buf = [0u8; 1024];

    loop {
        let num_read = input.read(&mut buf)?;
        if num_read == 0 {
            break;
        }

        let buf = &buf[..num_read];
        for output in &mut *outputs {
            output.write_all(buf)?;
        }
    }

    Ok(())
}

pub(crate) fn root_menu(config: &Config) -> Option<Menu> {
    if config.general.always_show_help.enabled {
        Some(Menu::Help)
    } else {
        None
    }
}

fn write_child_output_to_log(
    log_rwlock: &mut Arc<RwLock<CmdLogEntry>>,
    child: &mut Child,
    status: std::process::ExitStatus,
) -> Res<()> {
    let mut log = log_rwlock.write().unwrap();

    let CmdLogEntry::Cmd { args, out: out_log } = log.deref_mut() else {
        unreachable!("pending_cmd is always CmdLogEntry::Cmd variant");
    };

    drop(child.stdin.take());

    let mut out_bytes = vec![];
    log::debug!("Reading stderr");

    child
        .stderr
        .take()
        .unwrap()
        .read_to_end(&mut out_bytes)
        .map_err(Error::CouldntReadCmdOutput)?;

    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut out_bytes)
        .map_err(Error::CouldntReadCmdOutput)?;

    let out_string = String::from_utf8_lossy(&out_bytes).to_string();
    *out_log = Some(out_string.into());

    if !status.success() {
        return Err(Error::CmdBadExit(args.to_string(), status.code()));
    }

    Ok(())
}

type DefaultFn = Box<dyn Fn(&State) -> Option<String>>;

pub(crate) struct PromptParams {
    pub prompt: &'static str,
    pub create_default_value: DefaultFn,
    pub hide_menu: bool,
}

impl Default for PromptParams {
    fn default() -> Self {
        Self {
            prompt: "",
            create_default_value: Box::new(|_| None),
            hide_menu: true,
        }
    }
}
