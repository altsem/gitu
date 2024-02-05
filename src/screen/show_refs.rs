use ratatui::{
    style::Style,
    text::{Line, Span, Text},
};

use crate::{
    git,
    items::{Item, TargetData},
    theme::CURRENT_THEME,
    Res,
};

use super::Screen;

pub(crate) fn create() -> Res<Screen> {
    Screen::new(Box::new(move || {
        Ok(git::show_refs()?
            .into_iter()
            .map(|(local, remote, subject)| {
                let columns = [
                    Some(Span::styled(
                        local.clone(),
                        Style::new().fg(CURRENT_THEME.branch),
                    )),
                    (!remote.is_empty())
                        .then_some(Span::styled(remote, Style::new().fg(CURRENT_THEME.remote))),
                    Some(Span::raw(subject)),
                ]
                .into_iter()
                .flatten();

                let spans = itertools::intersperse(columns, Span::raw(" ")).collect::<Vec<_>>();

                Item {
                    id: local.clone().into(),
                    display: Text::from(vec![Line::from(spans)]),
                    depth: 0,
                    target_data: Some(TargetData::Ref(local.to_string())),
                    ..Default::default()
                }
            })
            .collect())
    }))
}
