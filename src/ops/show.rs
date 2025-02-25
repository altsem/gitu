use super::OpTrait;
use crate::{git::diff::first_diff_line, items::TargetData, screen, state::State, Action};
use core::str;
use std::{path::Path, process::Command, rc::Rc};

pub(crate) struct Show;
impl OpTrait for Show {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => goto_show_screen(r.clone()),
            Some(TargetData::File(u)) => editor(u.as_path(), None),
            Some(TargetData::Delta { diff, file_i }) => editor(
                Path::new(
                    str::from_utf8(&diff.text[diff.file_diffs[*file_i].header.new_file.clone()])
                        .unwrap(),
                ),
                None,
            ),
            Some(TargetData::Hunk {
                diff,
                file_i,
                hunk_i,
            }) => editor(
                Path::new(
                    str::from_utf8(&diff.text[diff.file_diffs[*file_i].header.new_file.clone()])
                        .unwrap(),
                ),
                Some(first_diff_line(&diff, *file_i, *hunk_i) as u32),
            ),
            Some(TargetData::Stash { id: _, commit }) => goto_show_screen(commit.clone()),
            _ => None,
        }
    }
    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Show".into()
    }
}

fn goto_show_screen(r: String) -> Option<Action> {
    Some(Rc::new(move |state, term| {
        state.close_menu();
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

fn editor(file: &Path, maybe_line: Option<u32>) -> Option<Action> {
    let file = file.to_str().unwrap().to_string();

    Some(Rc::new(move |state, term| {
        const EDITOR_VARS: [&str; 3] = ["VISUAL", "EDITOR", "GIT_EDITOR"];
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

        let cmd = parse_editor_command(&editor, &file, maybe_line);

        state.close_menu();
        state
            .run_cmd_interactive(term, cmd)
            .map_err(|err| format!("Couldn't open editor {} due to: {}", editor, err))?;

        state.screen_mut().update()
    }))
}

fn parse_editor_command(editor: &str, file: &str, maybe_line: Option<u32>) -> Command {
    let args = &editor.split_whitespace().collect::<Vec<_>>();
    let mut cmd = Command::new(args[0]);
    cmd.args(&args[1..]);

    let lower = args[0].to_lowercase();

    if let Some(line) = maybe_line {
        if lower.ends_with("vi")
            || lower.ends_with("vim")
            || lower.ends_with("nvim")
            || lower.ends_with("nano")
            || lower.ends_with("nvr")
        {
            cmd.args([&format!("+{}", line), file]);
        } else {
            cmd.args([&format!("{}:{}", file, line)]);
        }
    } else {
        cmd.args([file.to_string()]);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    #[test]
    fn parse_editor_command_test() {
        let cmd = super::parse_editor_command("/bin/nAnO -f", "README.md", Some(42));
        assert_eq!(cmd.get_program(), OsStr::new("/bin/nAnO"));
        assert_eq!(
            &cmd.get_args().collect::<Vec<_>>(),
            &["-f", "+42", "README.md"]
        );
    }
}
