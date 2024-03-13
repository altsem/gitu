use super::TargetOpTrait;
use crate::{items::TargetData, screen, Action};
use ratatui::backend::Backend;
use std::{path::Path, process::Command, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Show;
impl<B: Backend> TargetOpTrait<B> for Show {
    fn get_action(&self, target: TargetData) -> Option<Action<B>> {
        match target {
            TargetData::Commit(r) | TargetData::Branch(r) => goto_show_screen::<B>(r.clone()),
            TargetData::File(u) => editor(u.as_path(), None),
            TargetData::Delta(d) => editor(d.new_file.as_path(), None),
            TargetData::Hunk(h) => editor(h.new_file.as_path(), Some(h.first_diff_line())),
        }
    }
}

fn goto_show_screen<B: Backend>(r: String) -> Option<Action<B>> {
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

fn editor<B: Backend>(file: &Path, line: Option<u32>) -> Option<Action<B>> {
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
