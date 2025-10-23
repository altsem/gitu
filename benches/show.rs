use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use gitu::{cli::Commands, config, term::TermBackend};
use ratatui::{Terminal, backend::TestBackend};

fn show(c: &mut Criterion) {
    c.bench_function("show", |b| {
        let mut terminal = Terminal::new(TermBackend::Test {
            backend: TestBackend::new(80, 1000),
            events: vec![],
        })
        .unwrap();

        let args = gitu::cli::Args {
            command: Some(Commands::Show {
                reference: "f4de01c0a12794d7b42a77b2138aa64119b90ea5".into(),
            }),
            print: true,
            ..Default::default()
        };

        let config = Arc::new(config::init_config(args.config.clone()).unwrap());

        b.iter(|| {
            gitu::run(config.clone(), &args, &mut terminal).unwrap();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = show
}
criterion_main!(benches);
