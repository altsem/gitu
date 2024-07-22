use super::{Action, OpTrait};
use crate::{git::diff::Hunk, items::TargetData};
use derive_more::Display;
use std::{path::PathBuf, process::Command, rc::Rc};

#[derive(Display)]
#[display(fmt = "Discard")]
pub(crate) struct Discard;
impl OpTrait for Discard {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::Branch(branch)) => discard_branch(branch),
            Some(TargetData::File(file)) => clean_file(file),
            Some(TargetData::Delta(d)) => match d.status {
                git2::Delta::Added => remove_file(d.new_file),
                _ => checkout_file(d.old_file),
            },
            Some(TargetData::Hunk(h)) => discard_unstaged_patch(h),
            _ => return None,
        };

        Some(super::create_y_n_prompt(action, "Really discard?"))
    }

    fn is_target_op(&self) -> bool {
        true
    }
}

fn discard_branch(branch: String) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["branch", "-d"]);
        cmd.arg(&branch);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn clean_file(file: PathBuf) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["clean", "--force"]);
        cmd.arg(&file);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn remove_file(file: PathBuf) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["rm", "--force"]);
        cmd.arg(&file);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn checkout_file(file: PathBuf) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["checkout", "HEAD", "--"]);
        cmd.arg(&file);

        state.close_menu();
        state.run_cmd(term, &[], cmd)
    })
}

fn discard_unstaged_patch(h: Rc<Hunk>) -> Action {
    Rc::new(move |state, term| {
        let mut cmd = Command::new("git");
        cmd.args(["apply", "--reverse"]);

        state.close_menu();
        state.run_cmd(term, &h.format_patch().into_bytes(), cmd)
    })
}
