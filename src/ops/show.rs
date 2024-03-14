use super::TargetOpTrait;
use crate::{items::TargetData, screen, Action};
use derive_more::Display;
use std::{path::Path, process::Command, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Show")]
pub(crate) struct Show;
impl TargetOpTrait for Show {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => goto_show_screen(r.clone()),
            Some(TargetData::File(u)) => editor(u.as_path(), None),
            Some(TargetData::Delta(d)) => editor(d.new_file.as_path(), None),
            Some(TargetData::Hunk(h)) => editor(h.new_file.as_path(), Some(h.first_diff_line())),
            None => None,
        }
    }
}

fn goto_show_screen(r: String) -> Option<Action> {
    Some(Box::new(move |state, term| {
        state.screens.push(
            screen::show::create(
                Rc::clone(&state.config),
                Rc::clone(&state.repo),
                term.size()?,
                r.clone(),
            )
            .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}

fn editor(file: &Path, line: Option<u32>) -> Option<Action> {
    let file = file.to_str().unwrap().to_string();

    Some(Box::new(move |state, term| {
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
            .issue_subscreen_command(term, cmd)
            .map_err(|err| format!("Couldn't open editor {} due to: {}", editor, err))?;

        state.screen_mut().update()
    }))
}
