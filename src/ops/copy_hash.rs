use super::{Action, OpTrait};
use crate::{app::App, error::Error, target_data::TargetData};
use std::rc::Rc;

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

    fn display(&self, _app: &App) -> String {
        "Copy hash".into()
    }
}

fn copy_hash(r: String) -> Option<Action> {
    Some(Rc::new(move |app, _term| {
        app.close_menu();
        match &mut app.state.clipboard {
            Some(cb) => {
                cb.set_text(r.clone()).map_err(Error::Clipboard)?;
                app.display_info("Commit hash copied to clipboard");
            }
            None => app.display_error("Clipboard not available"),
        }
        Ok(())
    }))
}
