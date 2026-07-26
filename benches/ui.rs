use std::rc::Rc;
use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use git2::Repository;
use gitu::app::App;
use gitu::cli::{Args, Commands};
use gitu::config;
use gitu::term::TermBackend;
use ratatui::{backend::TestBackend, layout::Size};

const REFERENCE: &str = "f4de01c0a12794d7b42a77b2138aa64119b90ea5";

/// Times a redraw of an already-built screen, leaving repo access, diff
/// parsing and item creation out of the measurement.
fn bench_redraw(c: &mut Criterion, name: &str, size: Size) {
    c.bench_function(name, |b| {
        let mut term = TermBackend::Test {
            backend: TestBackend::new(size.width, size.height),
            events: vec![],
        };

        let args = Args {
            command: Some(Commands::Show {
                reference: REFERENCE.into(),
            }),
            ..Default::default()
        };

        let config = Arc::new(config::init_config(args.config.clone()).unwrap());
        let repo = Rc::new(Repository::open_from_env().unwrap());
        let mut app = App::create(repo, size, &args, config, false).unwrap();

        b.iter(|| app.redraw_now(&mut term).unwrap());
    });
}

fn ui(c: &mut Criterion) {
    // The whole diff at once, so that per-row costs dominate.
    bench_redraw(c, "ui/40x1000", Size::new(40, 1000));
}

criterion_group!(benches, ui);
criterion_main!(benches);
