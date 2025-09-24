use super::OpTrait;
use crate::{
    app::State,
    error::Error,
    item_data::{ItemData, RefKind},
    screen, Action,
};
use core::str;
use std::{path::Path, process::Command, rc::Rc};

pub(crate) struct Show;
impl OpTrait for Show {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        match target {
            ItemData::Commit { oid, .. }
            | ItemData::Reference {
                kind: RefKind::Tag(oid),
                ..
            }
            | ItemData::Reference {
                kind: RefKind::Branch(oid),
                ..
            } => goto_show_screen(oid.clone()),
            ItemData::File(u) => editor(u.as_path(), None),
            ItemData::Delta { diff, file_i } => editor(
                Path::new(&diff.text[diff.file_diffs[*file_i].header.new_file.clone()]),
                None,
            ),
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => editor(
                Path::new(&diff.text[diff.file_diffs[*file_i].header.new_file.clone()]),
                Some(diff.file_line_of_first_diff(*file_i, *hunk_i) as u32),
            ),
            ItemData::Stash { commit, .. } => goto_show_screen(commit.clone()),
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
    Some(Rc::new(move |app, term| {
        app.close_menu();
        app.state.screens.push(
            screen::show::create(
                Rc::clone(&app.state.config),
                Rc::clone(&app.state.repo),
                term.size().map_err(Error::Term)?,
                r.clone(),
            )
            .expect("Couldn't create screen"),
        );
        Ok(())
    }))
}

pub(crate) const EDITOR_VARS: [&str; 4] = ["GITU_SHOW_EDITOR", "VISUAL", "EDITOR", "GIT_EDITOR"];
fn editor(file: &Path, maybe_line: Option<u32>) -> Option<Action> {
    let file = file.to_str().unwrap().to_string();

    Some(Rc::new(move |app, term| {
        let configured_editor = EDITOR_VARS
            .into_iter()
            .find_map(|var| std::env::var(var).ok());

        let Some(editor) = configured_editor else {
            return Err(Error::NoEditorSet);
        };

        let cmd = if cfg!(windows) {
            parse_editor_command_windows(&editor, &file, maybe_line)
        } else {
            parse_editor_command(&editor, &file, maybe_line)
        };

        app.close_menu();
        app.run_cmd_interactive(term, cmd)?;

        app.screen_mut().update()
    }))
}

fn parse_editor_command(editor: &str, file: &str, maybe_line: Option<u32>) -> Command {
    let args = &editor.split_whitespace().collect::<Vec<_>>();
    let mut cmd = Command::new(args[0]);

    cmd.args(&args[1..]);
    cmd.args(line_args(file, maybe_line, args[0].to_lowercase()));
    cmd
}

fn parse_editor_command_windows(editor: &str, file: &str, maybe_line: Option<u32>) -> Command {
    let args = &editor.split_whitespace().collect::<Vec<_>>();
    let mut cmd = Command::new("cmd");
    cmd.arg("/C");
    cmd.arg(args[0]);

    cmd.args(&args[1..]);
    cmd.args(line_args(file, maybe_line, args[0].to_lowercase()));
    cmd
}

fn line_args(file: &str, maybe_line: Option<u32>, lower: String) -> Vec<String> {
    if let Some(line) = maybe_line {
        if lower.ends_with("vi")
            || lower.ends_with("vim")
            || lower.ends_with("nvim")
            || lower.ends_with("nano")
            || lower.ends_with("micro")
            || lower.ends_with("nvr")
        {
            vec![format!("+{line}"), file.to_string()]
        } else {
            vec![format!("{file}:{line}")]
        }
    } else {
        vec![file.to_string()]
    }
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

    #[test]
    fn parse_editor_command_test_windows() {
        let cmd = super::parse_editor_command_windows("/bin/nAnO -f", "README.md", Some(42));
        assert_eq!(cmd.get_program(), OsStr::new("cmd"));
        assert_eq!(
            &cmd.get_args().collect::<Vec<_>>(),
            &["/C", "/bin/nAnO", "-f", "+42", "README.md"]
        );
    }
}
