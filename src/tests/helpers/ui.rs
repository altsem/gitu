use crate::{
    app::App,
    cli::Args,
    config::{self, Config},
    error::Error,
    key_parser::parse_keys,
    term::{Term, TermBackend},
    tests::helpers::RepoTestContext,
};
use git2::Repository;
use ratatui::{backend::TestBackend, layout::Size, Terminal};
use std::{path::PathBuf, rc::Rc, time::Duration};
use temp_dir::TempDir;
use termwiz::input::{InputEvent, KeyEvent};

use self::buffer::TestBuffer;

mod buffer;

#[macro_export]
macro_rules! snapshot {
    ($ctx:expr, $keys:expr) => {{
        let mut ctx = $ctx;
        let mut state = ctx.init_app();
        ctx.update(&mut state, keys($keys));
        insta::assert_snapshot!(ctx.redact_buffer());
        state
    }};
}

pub struct TestContext {
    pub term: Term,
    pub dir: TempDir,
    pub remote_dir: TempDir,
    pub size: Size,
    config: Rc<Config>,
}

impl TestContext {
    pub fn setup_init() -> Self {
        let size = Size::new(80, 20);
        let term = Terminal::new(TermBackend::Test {
            backend: TestBackend::new(size.width, size.height),
            events: vec![],
        })
        .unwrap();
        let repo_ctx = RepoTestContext::setup_init();
        Self {
            term,
            dir: repo_ctx.dir,
            remote_dir: repo_ctx.remote_dir,
            size,
            config: Rc::new(config::init_test_config().unwrap()),
        }
    }

    pub fn setup_clone() -> Self {
        let size = Size::new(80, 20);
        let term = Terminal::new(TermBackend::Test {
            backend: TestBackend::new(size.width, size.height),
            events: vec![],
        })
        .unwrap();
        let repo_ctx = RepoTestContext::setup_clone();
        Self {
            term,
            dir: repo_ctx.dir,
            remote_dir: repo_ctx.remote_dir,
            size,
            config: Rc::new(config::init_test_config().unwrap()),
        }
    }

    pub fn config(&mut self) -> &mut Config {
        Rc::get_mut(&mut self.config).unwrap()
    }

    pub fn init_app(&mut self) -> App {
        self.init_app_at_path(self.dir.path().to_path_buf())
    }

    pub fn init_app_at_path(&mut self, path: PathBuf) -> App {
        let mut app = App::create(
            Rc::new(Repository::open(path).unwrap()),
            self.size,
            &Args::default(),
            Rc::clone(&self.config),
            false,
        )
        .unwrap();

        app.redraw_now(&mut self.term).unwrap();
        app
    }

    pub fn update(&mut self, app: &mut App, new_events: Vec<InputEvent>) {
        let TermBackend::Test { events, .. } = self.term.backend_mut() else {
            unreachable!();
        };

        events.extend(new_events.into_iter().rev());

        let result = app.run(&mut self.term, Duration::ZERO);
        assert!(app.state.quit || matches!(result, Err(Error::NoMoreEvents)));
    }

    pub fn redact_buffer(&self) -> String {
        let TermBackend::Test { backend, .. } = self.term.backend() else {
            unreachable!();
        };
        let mut debug_output = format!("{:?}", TestBuffer(backend.buffer()));

        redact_temp_dir(&self.dir, &mut debug_output);
        redact_temp_dir(&self.remote_dir, &mut debug_output);

        debug_output
    }
}

fn redact_temp_dir(temp_dir: &TempDir, debug_output: &mut String) {
    let text = temp_dir.path().to_str().unwrap();
    *debug_output = debug_output.replace(text, &" ".repeat(text.len()));
}

pub fn keys(input: &str) -> Vec<InputEvent> {
    let ("", keys) = parse_keys(input).unwrap() else {
        unreachable!();
    };

    keys.into_iter()
        .map(|(modifiers, key)| InputEvent::Key(KeyEvent { key, modifiers }))
        .collect()
}
