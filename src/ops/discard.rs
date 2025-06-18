use gitu_diff::Status;

use super::{confirm, Action, OpTrait};
use crate::{
    app::State,
    config::ConfirmDiscardOption,
    git::diff::{Diff, PatchMode},
    gitu_diff,
    item_data::{ItemData, RefKind},
};
use std::{path::PathBuf, process::Command, rc::Rc};

pub(crate) struct Discard;
impl OpTrait for Discard {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::Reference(reference) => {
                let branch = match reference {
                    RefKind::Tag(tag) => tag,
                    RefKind::Branch(branch) => branch,
                    // FIXME is this correct?
                    RefKind::Remote(_) => unreachable!("cannot discard remotes"),
                };
                discard_branch(branch.clone())
            }
            ItemData::File(file) => clean_file(file.clone()),
            ItemData::Delta { diff, file_i } => match diff.file_diffs[*file_i].header.status {
                Status::Added => {
                    remove_file(diff.text[diff.file_diffs[*file_i].header.new_file.clone()].into())
                }
                Status::Renamed => rename_file(
                    diff.text[diff.file_diffs[*file_i].header.new_file.clone()].into(),
                    diff.text[diff.file_diffs[*file_i].header.old_file.clone()].into(),
                ),
                _ => checkout_file(
                    diff.text[diff.file_diffs[*file_i].header.old_file.clone()].into(),
                ),
            },
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => discard_unstaged_patch(Rc::clone(diff), *file_i, *hunk_i),
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
            } => discard_unstaged_line(Rc::clone(diff), *file_i, *hunk_i, *line_i),
            _ => return None,
        };

        Some(action)
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Discard".into()
    }
}

fn discard_branch(branch: String) -> Action {
    Rc::new(move |app, term| {
        confirm(app, term, "Really discard? (y or n)")?;
        super::branch::delete(app, term, &branch)
    })
}

fn clean_file(file: PathBuf) -> Action {
    Rc::new(move |app, term| {
        if app.state.config.general.confirm_discard <= ConfirmDiscardOption::File {
            confirm(app, term, "Really discard? (y or n)")?;
        }

        let mut cmd = Command::new("git");
        cmd.args(["clean", "--force"]);
        cmd.arg(&file);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn rename_file(src: PathBuf, dest: PathBuf) -> Action {
    Rc::new(move |app, term| {
        if app.state.config.general.confirm_discard <= ConfirmDiscardOption::File {
            confirm(app, term, "Really discard? (y or n)")?;
        }

        let mut cmd = Command::new("git");
        cmd.args(["mv", "--force"]);
        cmd.arg(&src);
        cmd.arg(&dest);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn remove_file(file: PathBuf) -> Action {
    Rc::new(move |app, term| {
        if app.state.config.general.confirm_discard <= ConfirmDiscardOption::File {
            confirm(app, term, "Really discard? (y or n)")?;
        }

        let mut cmd = Command::new("git");
        cmd.args(["rm", "--force"]);
        cmd.arg(&file);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn checkout_file(file: PathBuf) -> Action {
    Rc::new(move |app, term| {
        if app.state.config.general.confirm_discard <= ConfirmDiscardOption::File {
            confirm(app, term, "Really discard? (y or n)")?;
        }

        let mut cmd = Command::new("git");
        cmd.args(["checkout", "HEAD", "--"]);
        cmd.arg(&file);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn discard_unstaged_patch(diff: Rc<Diff>, file_i: usize, hunk_i: usize) -> Action {
    Rc::new(move |app, term| {
        if app.state.config.general.confirm_discard <= ConfirmDiscardOption::Hunk {
            confirm(app, term, "Really discard? (y or n)")?;
        }

        let mut cmd = Command::new("git");
        cmd.args(["apply", "--reverse"]);

        app.close_menu();
        app.run_cmd(
            term,
            &diff.format_hunk_patch(file_i, hunk_i).into_bytes(),
            cmd,
        )
    })
}

fn discard_unstaged_line(diff: Rc<Diff>, file_i: usize, hunk_i: usize, line_i: usize) -> Action {
    Rc::new(move |app, term| {
        if app.state.config.general.confirm_discard <= ConfirmDiscardOption::Line {
            confirm(app, term, "Really discard? (y or n)")?;
        }

        let mut cmd = Command::new("git");
        cmd.args(["apply", "--reverse", "--recount"]);

        let input = diff
            .format_line_patch(file_i, hunk_i, line_i..(line_i + 1), PatchMode::Reverse)
            .into_bytes();

        app.close_menu();
        app.run_cmd(term, &input, cmd)
    })
}
