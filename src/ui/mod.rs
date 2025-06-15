use std::cell::RefCell;
use std::rc::Rc;

use termwiz::color::ColorAttribute;
use termwiz::surface::Change;
use widgets::layout::{ChildOrientation, Constraints, Dimension, DimensionSpec, VerticalAlignment};
use widgets::{RenderArgs, UpdateArgs, Widget, WidgetEvent};

use crate::app::State;
use crate::screen::Screen;

pub(crate) mod widgets;

pub(crate) struct Root {}

impl Widget<State> for Root {
    fn render(&mut self, _args: &mut RenderArgs<State>) {}

    fn get_size_constraints(&self) -> Constraints {
        Constraints {
            valign: VerticalAlignment::Bottom,
            child_orientation: ChildOrientation::Vertical,
            ..Default::default()
        }
    }
}

pub(crate) struct ScreenWindow(pub Rc<RefCell<Vec<Screen>>>);

impl Widget<State> for ScreenWindow {
    fn render(&mut self, args: &mut RenderArgs<State>) {
        let mut screens = self.0.borrow_mut();
        let Some(screen) = screens.last_mut() else {
            return;
        };

        screen.render(args);
    }

    fn get_size_constraints(&self) -> Constraints {
        let mut screens = self.0.borrow_mut();
        let Some(screen) = screens.last_mut() else {
            return Constraints::default();
        };

        screen.get_size_constraints()
    }

    fn process_event(&mut self, event: &WidgetEvent, args: &mut UpdateArgs) -> bool {
        let mut screens = self.0.borrow_mut();
        let Some(screen) = screens.last_mut() else {
            return false;
        };

        screen.process_event(event, args)
    }
}

pub(crate) struct Menu {}

impl Widget<State> for Menu {
    fn render(&mut self, args: &mut RenderArgs<State>) {
        args.surface
            .add_change(Change::ClearScreen(ColorAttribute::PaletteIndex(8)));
        // TODO
    }

    fn get_size_constraints(&self) -> Constraints {
        // TODO auto-size depending on amount of lines in the menu
        Constraints {
            height: Dimension {
                spec: DimensionSpec::Fixed(10),
                maximum: Some(10),
                minimum: Some(10),
            },
            ..Default::default()
        }
    }
}
