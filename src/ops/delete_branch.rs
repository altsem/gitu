use super::{create_prompt_with_default, create_y_n_prompt, selected_rev, Action, OpTrait};
use crate::{error::Error, items::TargetData, menu::arg::Arg, state::State, term::Term, Res};
use std::{process::Command, rc::Rc};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![]
}

pub(crate) struct DeleteBranch;
impl OpTrait for DeleteBranch {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target {
            Some(TargetData::Branch(branch)) => {
                let branch = branch.clone();
                Some(create_y_n_prompt(
                    Rc::new(move |state, term| delete_branch(state, term, &branch)),
                    "Delete this branch?",
                ))
            }
            _ => Some(create_prompt_with_default(
                "Delete branch",
                delete_branch,
                selected_rev,
                true,
            )),
        }
    }

    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Delete branch".into()
    }
}

fn delete_branch(state: &mut State, term: &mut Term, branch: &str) -> Res<()> {
    if branch.is_empty() {
        return Err(Error::NotOnBranch);
    }

    // Don't allow deleting current branch
    let current_branch = state
        .repo
        .head()
        .map_err(Error::GetHead)?
        .shorthand()
        .unwrap_or("")
        .to_string();

    if current_branch == branch {
        state.display_error("Cannot delete the current branch".to_string());
        return Ok(());
    }

    let delete_flag = "-d";

    state.close_menu();

    let mut cmd = Command::new("git");
    cmd.args(["branch", delete_flag, branch]);
    state.run_cmd(term, &[], cmd)?;

    Ok(())
}
