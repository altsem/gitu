use super::Screen;
use crate::{
    git,
    items::{Item, TargetData},
    theme::CURRENT_THEME,
    Config, Res,
};
use ratatui::{
    prelude::Rect,
    style::{Style, Stylize},
    text::{Line, Span, Text},
};

pub(crate) fn create(config: &Config, size: Rect) -> Res<Screen> {
    let path_buf = config.dir.clone();
    Screen::new(
        size,
        Box::new(move || {
            // TODO Replace with libgit2
            Ok(git::show_refs(&path_buf)?
                .into_iter()
                .map(|(local, remote, subject)| {
                    let columns = [
                        Some(Span::styled(
                            local.clone(),
                            Style::new().fg(CURRENT_THEME.branch).bold(),
                        )),
                        (!remote.is_empty()).then_some(Span::styled(
                            remote,
                            Style::new().fg(CURRENT_THEME.remote).bold(),
                        )),
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
        }),
    )
}
