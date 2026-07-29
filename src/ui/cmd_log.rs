use std::sync::{Arc, RwLock};

use crate::style::Style;

use crate::cmd_log::{CmdLog, CmdLogEntry};
use crate::config::Config;
use crate::ui::layout::opts;
use crate::ui::{DASHES, UiTree, layout_line, layout_span, repeat_chars};

pub(crate) fn layout_cmd_log<'a>(
    layout: &mut UiTree<'a>,
    log: &CmdLog,
    config: &Config,
    width: usize,
) {
    if log.is_empty() {
        return;
    }

    repeat_chars(layout, width, DASHES, Style::from(&config.style.separator));

    layout.col(opts(), |layout| {
        for entry in &log.entries {
            layout_entry(layout, entry, config);
        }
    });
}

/// The entry is read under a lock, so its spans have to be owned.
fn layout_entry<'a>(layout: &mut UiTree<'a>, entry: &Arc<RwLock<CmdLogEntry>>, config: &Config) {
    match &*entry.read().unwrap() {
        CmdLogEntry::Cmd { args, out } => {
            layout.row(opts(), |layout| {
                layout_span(
                    layout,
                    (
                        if out.is_some() { "$ " } else { "Running: " }.into(),
                        Style::from(&config.style.info_msg),
                    ),
                );
                layout_span(
                    layout,
                    (args.to_string().into(), Style::from(&config.style.command)),
                );
            });

            for line in out.iter().flat_map(|out| out.lines()) {
                layout_line(layout, line.to_string().into(), Style::new());
            }
        }
        CmdLogEntry::Error(err) => layout_line(
            layout,
            format!("! {err}").into(),
            Style::from(&config.style.error_msg),
        ),
        CmdLogEntry::Info(msg) => layout_line(
            layout,
            format!("> {msg}").into(),
            Style::from(&config.style.info_msg),
        ),
    }
}
