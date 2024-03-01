use std::{iter, rc::Rc};

use super::Screen;
use crate::{
    items::{Item, TargetData},
    theme::CURRENT_THEME,
    Res,
};
use git2::{BranchType, Repository};
use ratatui::{
    prelude::Rect,
    style::Stylize,
    text::{Line, Span, Text},
};

pub(crate) fn create(repo: Rc<Repository>, size: Rect) -> Res<Screen> {
    Screen::new(
        size,
        Box::new(move || {
            Ok(iter::once(Item {
                id: "branches".into(),
                display: Text::from("Branches".to_string().fg(CURRENT_THEME.section)),
                section: true,
                depth: 0,
                ..Default::default()
            })
            .chain(
                repo.branches(Some(BranchType::Local))?
                    .filter_map(Result::ok)
                    .map(|(branch, _branch_type)| {
                        let name = Span::raw(branch.name().unwrap().unwrap().to_string())
                            .fg(CURRENT_THEME.branch);

                        let upstream_name = if let Ok(upstream) = branch.upstream() {
                            if let Ok(Some(name)) = upstream.name() {
                                Span::raw(name.to_string()).fg(CURRENT_THEME.remote)
                            } else {
                                Span::raw("")
                            }
                        } else {
                            Span::raw("")
                        };

                        Item {
                            id: name.clone().content,
                            display: Line::from(vec![name.clone(), Span::raw(" "), upstream_name])
                                .into(),
                            depth: 1,
                            target_data: Some(TargetData::Branch(name.content.into())),
                            ..Default::default()
                        }
                    }),
            )
            .collect())
        }),
    )
}
