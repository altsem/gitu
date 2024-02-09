use super::Screen;
use crate::{
    diff::Diff,
    git,
    items::{self, Item},
    theme::CURRENT_THEME,
    Config, Res,
};
use ansi_to_tui::IntoText;
use ratatui::{
    style::{Style, Stylize},
    text::Text,
};
use std::iter;

pub(crate) fn create(config: &Config, status: bool) -> Res<Screen> {
    let config = config.clone();
    Screen::new(Box::new(move || {
        let untracked = git::status(&config.dir)?
            .files
            .iter()
            .filter(|file| file.is_untracked())
            .map(|file| Item {
                id: file.path.clone().into(),
                display: Text::styled(
                    file.path.clone(),
                    Style::new().fg(CURRENT_THEME.unstaged_file).bold(),
                ),
                depth: 1,
                target_data: Some(items::TargetData::File(file.path.clone())),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let unmerged = git::status(&config.dir)?
            .files
            .iter()
            .filter(|file| file.is_unmerged())
            .map(|file| Item {
                id: file.path.clone().into(),
                display: Text::styled(
                    file.path.clone(),
                    Style::new().fg(CURRENT_THEME.unmerged_file).bold(),
                ),
                depth: 1,
                target_data: Some(items::TargetData::File(file.path.clone())),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let items = status
            .then_some(Item {
                id: "status".into(),
                display: git::status_simple(&config.dir)?
                    .into_text()
                    .expect("Error parsing status ansi"),
                unselectable: true,
                ..Default::default()
            })
            .into_iter()
            .chain(if untracked.is_empty() {
                None
            } else {
                Some(Item {
                    id: "untracked".into(),
                    display: Text::styled(
                        "\nUntracked files".to_string(),
                        Style::new().fg(CURRENT_THEME.section).bold(),
                    ),
                    section: true,
                    depth: 0,
                    ..Default::default()
                })
            })
            .chain(untracked)
            .chain(if unmerged.is_empty() {
                None
            } else {
                Some(Item {
                    id: "unmerged".into(),
                    display: Text::styled(
                        "\nUnmerged".to_string(),
                        Style::new().fg(CURRENT_THEME.section).bold(),
                    ),
                    section: true,
                    depth: 0,
                    ..Default::default()
                })
            })
            .chain(unmerged)
            .chain(create_status_section_items(
                "\nUnstaged changes",
                &git::diff_unstaged(&config.dir)?,
            ))
            .chain(create_status_section_items(
                "\nStaged changes",
                &git::diff_staged(&config.dir)?,
            ))
            .chain(create_log_section_items(
                "\nRecent commits",
                &git::log_recent(&config.dir)?,
            ))
            .collect();

        Ok(items)
    }))
}

// fn format_branch_status(status: &BranchStatus) -> String {
//     let Some(ref remote) = status.remote else {
//         return format!("On branch {}.", status.local);
//     };

//     if status.ahead == 0 && status.behind == 0 {
//         format!(
//             "On branch {}\nYour branch is up to date with '{}'.",
//             status.local, remote
//         )
//     } else if status.ahead > 0 && status.behind == 0 {
//         format!(
//             "On branch {}\nYour branch is ahead of '{}' by {} commit.",
//             status.local, remote, status.ahead
//         )
//     } else if status.ahead == 0 && status.behind > 0 {
//         format!(
//             "On branch {}\nYour branch is behind '{}' by {} commit.",
//             status.local, remote, status.behind
//         )
//     } else {
//         format!("On branch {}\nYour branch and '{}' have diverged,\nand have {} and {} different commits each, respectively.", status.local, remote, status.ahead, status.behind)
//     }
// }

fn create_status_section_items<'a>(
    header: &str,
    diff: &'a Diff,
) -> impl Iterator<Item = Item> + 'a {
    if diff.deltas.is_empty() {
        None
    } else {
        Some(Item {
            id: header.to_string().into(),
            display: Text::styled(
                format!("{} ({})", header, diff.deltas.len()),
                Style::new().fg(CURRENT_THEME.section).bold(),
            ),
            section: true,
            depth: 0,
            ..Default::default()
        })
    }
    .into_iter()
    .chain(items::create_diff_items(diff, &1))
}

fn create_log_section_items<'a>(header: &str, log: &'a str) -> impl Iterator<Item = Item> + 'a {
    iter::once(Item {
        id: header.to_string().into(),
        display: Text::styled(
            header.to_string(),
            Style::new().fg(CURRENT_THEME.section).bold(),
        ),
        section: true,
        depth: 0,
        ..Default::default()
    })
    .chain(items::create_log_items(log))
}
