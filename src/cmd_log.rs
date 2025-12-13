use crate::config::Config;
use itertools::Itertools;
use ratatui::text::{Line, Span};
use ratatui::text::Text;
use std::borrow::Cow;
use std::iter;
use std::process::Command;
use std::sync::Arc;
use std::sync::RwLock;

pub(crate) struct CmdLog {
    pub(crate) entries: Vec<Arc<RwLock<CmdLogEntry>>>,
}

impl CmdLog {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn push_cmd(&mut self, cmd: &Command) -> Arc<RwLock<CmdLogEntry>> {
        let value = Arc::new(RwLock::new(CmdLogEntry::Cmd {
            args: command_args(cmd),
            out: None,
        }));

        self.entries.push(Arc::clone(&value));
        value
    }

    pub fn push_cmd_with_output(
        &mut self,
        cmd: &Command,
        out: Cow<'static, str>,
    ) -> Arc<RwLock<CmdLogEntry>> {
        let value = Arc::new(RwLock::new(CmdLogEntry::Cmd {
            args: command_args(cmd),
            out: Some(out),
        }));

        self.entries.push(Arc::clone(&value));
        value
    }

    pub fn push(&mut self, entry: CmdLogEntry) {
        self.entries.push(Arc::new(RwLock::new(entry)));
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn format_log(&self, config: &Config) -> Text<'static> {
        Text::from(
            self.entries
                .iter()
                .flat_map(|cmd| format_log_entry(config, cmd))
                .collect::<Vec<_>>(),
        )
    }
}

pub(crate) fn command_args(cmd: &Command) -> Cow<'static, str> {
    iter::once(cmd.get_program().to_string_lossy())
        .chain(cmd.get_args().map(|arg| arg.to_string_lossy()))
        .join(" ")
        .into()
}

pub(crate) fn format_log_entry<'a>(
    config: &Config,
    log: &Arc<RwLock<CmdLogEntry>>,
) -> Vec<Line<'a>> {
    match &*log.read().unwrap() {
        CmdLogEntry::Cmd { args, out } => [Line::from(vec![
            Span::styled(
                if out.is_some() { "$ " } else { "Running: " },
                &config.style.info_msg,
            ),
            Span::styled(args.to_string(), &config.style.command),
        ])]
        .into_iter()
        .chain(out.iter().flat_map(|out| {
            if out.is_empty() {
                vec![]
            } else {
                Text::raw(out.to_string()).lines
            }
        }))
        .collect::<Vec<_>>(),
        CmdLogEntry::Error(err) => {
            vec![Line::styled(format!("! {err}"), &config.style.error_msg)]
        }
        CmdLogEntry::Info(msg) => {
            vec![Line::styled(format!("> {msg}"), &config.style.info_msg)]
        }
    }
}

pub(crate) enum CmdLogEntry {
    Cmd {
        args: Cow<'static, str>,
        out: Option<Cow<'static, str>>,
    },
    Error(String),
    Info(String),
}
