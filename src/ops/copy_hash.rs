use super::{Action, OpTrait};
use crate::items::TargetData;
use derive_more::Display;
use std::rc::Rc;

#[derive(Display)]
#[display(fmt = "Copy hash")]
pub(crate) struct CopyHash;
impl OpTrait for CopyHash {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target {
            Some(TargetData::Commit(r)) => copy_hash(r.clone()),
            _ => None,
        }
    }
    fn is_target_op(&self) -> bool {
        true
    }
}

fn copy_hash(r: String) -> Option<Action> {
    Some(Rc::new(move |state, _term| {
        state
            .clipboard
            .set_text(r.clone())?;
        state.display_info("Commit hash copied to clipboard".to_owned());
        Ok(())
    }))
}
