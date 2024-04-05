use super::{cmd, cmd_arg, unstage::unstage_file_cmd, Action, OpTrait};
use crate::{items::TargetData, state::State, term::Term};
use derive_more::Display;
use std::{ffi::OsStr, process::Command, rc::Rc};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Discard")]
pub(crate) struct Discard;
impl OpTrait for Discard {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target.cloned() {
            Some(TargetData::Branch(r)) => cmd_arg(discard_branch, r.into()),
            Some(TargetData::File(f)) => cmd_arg(clean_file_cmd, f.into()),
            Some(TargetData::Delta(d)) => {
                match d.status {
                    git2::Delta::Added => Rc::new(move |state: &mut State, term: &mut Term| {
                        let file = d.new_file.as_os_str();
                        state.run_cmd(term, &[], unstage_file_cmd(file))?;
                        state.run_cmd(term, &[], clean_file_cmd(file))
                    }),
                    _ => {
                        if d.old_file == d.new_file {
                            cmd_arg(checkout_file_cmd, d.old_file.into())
                        } else {
                            // TODO Discard file move
                            return None;
                        }
                    }
                }
            }
            Some(TargetData::Hunk(h)) => {
                cmd(h.format_patch().into_bytes(), discard_unstaged_patch_cmd)
            }
            _ => return None,
        };

        Some(super::create_y_n_prompt(action, "Really discard?"))
    }

    fn is_target_op(&self) -> bool {
        true
    }
}

fn discard_unstaged_patch_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["apply", "--reverse"]);
    cmd
}

fn clean_file_cmd(file: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["clean", "--force"]);
    cmd.arg(file);
    cmd
}

fn checkout_file_cmd(file: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["checkout", "HEAD", "--"]);
    cmd.arg(file);
    cmd
}

fn discard_branch(branch: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["branch", "-d"]);
    cmd.arg(branch);
    cmd
}
