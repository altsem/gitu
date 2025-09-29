use crate::app::State;
use crate::screen;
use layout::LayoutTree;
use layout::OPTS;
use ratatui::Frame;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use tui_prompts::State as _;

pub(crate) mod layout;
mod menu;

const CARET: &str = "\u{2588}";

pub(crate) fn ui(frame: &mut Frame, state: &mut State, layout: &mut LayoutTree<Span>) {
    layout.clear();

    layout.stacked(OPTS, |layout| {
        screen::layout_screen(
            layout,
            frame.area().as_size(),
            state.screens.last().unwrap(),
        );

        layout.vertical(OPTS.align_end(), |layout| {
            menu::layout_menu(layout, state);
            layout_command_log(layout, state);
            layout_prompt(layout, state);
        });
    });

    layout.compute([frame.area().width, frame.area().height]);

    for span in layout.iter() {
        let area = Rect {
            x: span.pos[0],
            y: span.pos[1],
            width: span.size[0],
            height: span.size[1],
        };

        frame.render_widget(span.data, area);
    }

    state.screens.last_mut().unwrap().size = frame.area().as_size();
}

fn layout_command_log(layout: &mut LayoutTree<Span<'_>>, state: &mut State) {
    if !state.current_cmd_log.is_empty() {
        layout_text(layout, state.current_cmd_log.format_log(&state.config));
    }
}

fn layout_prompt(layout: &mut LayoutTree<Span>, state: &mut State) {
    let Some(ref prompt_data) = state.prompt.data else {
        return;
    };

    let prompt_symbol = state.prompt.state.status().symbol();

    layout.horizontal(OPTS, |layout| {
        let line = Line::from(vec![
            prompt_symbol,
            " ".into(),
            Span::raw(prompt_data.prompt_text.to_string()),
            " › ".cyan().dim(),
            Span::raw(state.prompt.state.value().to_string()),
            Span::raw(CARET),
        ]);

        layout_line(layout, line);
    });
}

pub(crate) fn layout_text<'a>(layout: &mut LayoutTree<Span<'a>>, text: Text<'a>) {
    layout.vertical(OPTS, |layout| {
        for line in text {
            layout_line(layout, line);
        }
    });
}

pub(crate) fn layout_line<'a>(layout: &mut LayoutTree<Span<'a>>, line: Line<'a>) {
    layout.horizontal(OPTS, |layout| {
        for span in line {
            layout_span(layout, span);
        }
    });
}

pub(crate) fn layout_span<'a>(layout: &mut LayoutTree<Span<'a>>, span: Span<'a>) {
    layout.leaf_with_size(span.clone(), [span.width() as u16, 1]);
}
