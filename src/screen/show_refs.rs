use std::{iter, rc::Rc};

use super::Screen;
use crate::{
    config::Config,
    items::{Item, TargetData},
    Res,
};
use git2::{BranchType, Repository};
use ratatui::{
    prelude::Rect,
    text::{Line, Span},
};

pub(crate) fn create(config: Rc<Config>, repo: Rc<Repository>, size: Rect) -> Res<Screen> {
    Screen::new(
        Rc::clone(&config),
        size,
        Box::new(move || {
            let color = &config.color;
            let head = repo.head().ok();

            Ok(iter::once(Item {
                id: "branches".into(),
                display: Line::styled("Branches".to_string(), &color.section),
                section: true,
                depth: 0,
                ..Default::default()
            })
            .chain(
                repo.branches(Some(BranchType::Local))?
                    .filter_map(Result::ok)
                    .map(|(branch, _branch_type)| {
                        let name = Span::styled(
                            branch.name().unwrap().unwrap().to_string(),
                            &color.branch,
                        );

                        let prefix = Span::raw(
                            if branch.get().name() == head.as_ref().and_then(|h| h.name()) {
                                "* "
                            } else {
                                "  "
                            },
                        );

                        let upstream_name = if let Ok(upstream) = branch.upstream() {
                            if let Ok(Some(name)) = upstream.name() {
                                Span::styled(name.to_string(), &color.remote)
                            } else {
                                Span::raw("")
                            }
                        } else {
                            Span::raw("")
                        };

                        Item {
                            id: name.clone().content,
                            display: Line::from(vec![
                                prefix,
                                name.clone(),
                                Span::raw("   "),
                                upstream_name,
                            ]),
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
