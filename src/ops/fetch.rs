use super::OpTrait;
use crate::{state::State, term::Term, Res};
use std::process::Command;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct FetchAll;
impl OpTrait for FetchAll {
    fn trigger(&self, state: &mut State, term: &mut Term) -> Res<()> {
        let mut cmd = Command::new("git");
        cmd.args(["fetch", "--all"]);

        state.run_external_cmd(term, &[], cmd)?;
        Ok(())
    }
}
