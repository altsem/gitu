use itertools::Itertools;
use ratatui::text::Text;

use crate::{
    git,
    items::{Item, TargetData},
};

use super::Screen;

pub(crate) fn create() -> Screen {
    Screen::new(Box::new(move || {
        git::show_refs()
            .into_iter()
            .map(|(local, remote, subject)| Item {
                id: local.clone().into(),
                display: Text::raw(
                    [local.clone(), remote, subject]
                        .iter()
                        .filter(|span| !span.is_empty())
                        .join(" "),
                ),
                depth: 0,
                target_data: Some(TargetData::Ref(local.to_string())),
                ..Default::default()
            })
            .collect()
    }))
}
