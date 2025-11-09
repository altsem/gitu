use super::{Action, OpTrait, confirm};
use crate::{
    Res,
    app::{App, State},
    config::ConfirmDiscardOption,
    git::diff::{DiffType, PatchMode},
    item_data::{ItemData, RefKind},
    term::Term,
};
use std::{path::PathBuf, process::Command, rc::Rc};

pub(crate) struct Discard;
impl OpTrait for Discard {
    fn get_action(&self, target: &ItemData) -> Option<Action> {
        let action = match target {
            ItemData::Reference {
                kind: RefKind::Branch(branch),
                ..
            } => discard_branch(branch.clone()),
            ItemData::Untracked(file) => clean_file(file.clone()),
            ItemData::Delta { diff, file_i } => {
                let patch = diff.format_file_patch(*file_i);
                match diff.diff_type {
                    DiffType::WorkdirToIndex => reverse_worktree(patch),
                    DiffType::IndexToTree => reverse_index_and_worktree(patch),
                    DiffType::TreeToTree => reverse_index_and_worktree(patch),
                }
            }
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => {
                let patch = diff.format_hunk_patch(*file_i, *hunk_i);
                match diff.diff_type {
                    DiffType::WorkdirToIndex => reverse_worktree(patch),
                    DiffType::IndexToTree => reverse_index_and_worktree(patch),
                    DiffType::TreeToTree => reverse_index_and_worktree(patch),
                }
            }
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_i,
                ..
            } => {
                let patch = diff.format_line_patch(
                    *file_i,
                    *hunk_i,
                    *line_i..(line_i + 1),
                    PatchMode::Reverse,
                );

                match diff.diff_type {
                    DiffType::WorkdirToIndex => reverse_worktree(patch),
                    DiffType::IndexToTree => reverse_index_and_worktree(patch),
                    DiffType::TreeToTree => reverse_index_and_worktree(patch),
                }
            }
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
        confirm_discard(app, term)?;

        let mut cmd = Command::new("git");
        cmd.args(["clean", "--force"]);
        cmd.arg(&file);

        app.close_menu();
        app.run_cmd(term, &[], cmd)
    })
}

fn reverse_worktree(patch: String) -> Action {
    let patch_bytes = patch.into_bytes();

    Rc::new(move |app, term| {
        confirm_discard(app, term)?;

        let mut cmd = Command::new("git");
        cmd.args(["apply", "--reverse", "--recount"]);

        app.close_menu();
        app.run_cmd(term, &patch_bytes, cmd)
    })
}

fn reverse_index_and_worktree(patch: String) -> Action {
    let patch_bytes = patch.into_bytes();

    Rc::new(move |app, term| {
        confirm_discard(app, term)?;

        let mut cmd = Command::new("git");
        cmd.args(["apply", "--reverse", "--index", "--recount"]);

        app.close_menu();
        app.run_cmd(term, &patch_bytes, cmd)
    })
}

fn confirm_discard(app: &mut App, term: &mut Term) -> Res<()> {
    if app.state.config.general.confirm_discard <= ConfirmDiscardOption::File {
        confirm(app, term, "Really discard? (y or n)")?;
    }
    Ok(())
}
