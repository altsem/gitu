use criterion::{criterion_group, criterion_main, Criterion};
use gitu::{cli::Commands, term::TermBackend};
use ratatui::{backend::TestBackend, Terminal};

fn show(c: &mut Criterion) {
    c.bench_function("show", |b| {
        let mut terminal = Terminal::new(TermBackend::Test {
            backend: TestBackend::new(80, 1000),
            events: vec![],
        })
        .unwrap();
        b.iter(|| {
            gitu::run(
                &gitu::cli::Args {
                    command: Some(Commands::Show {
                        reference: "f4de01c0a12794d7b42a77b2138aa64119b90ea5".into(),
                    }),
                    print: true,
                    ..Default::default()
                },
                &mut terminal,
            )
            .unwrap();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = show
}
criterion_main!(benches);
