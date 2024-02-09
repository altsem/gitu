use criterion::{criterion_group, criterion_main, Criterion};
use gitu::cli::Commands;
use pprof::criterion::{Output, PProfProfiler};
use ratatui::{backend::TestBackend, Terminal};

fn show(c: &mut Criterion) {
    c.bench_function("show", |b| {
        let mut terminal = Terminal::new(TestBackend::new(80, 1000)).unwrap();
        b.iter(|| {
            gitu::run(
                gitu::cli::Args {
                    command: Some(Commands::Show {
                        git_show_args: vec!["f4de01c0a12794d7b42a77b2138aa64119b90ea5".into()],
                    }),
                    status: false,
                    exit_immediately: true,
                },
                &mut terminal,
            )
            .unwrap();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = show
}
criterion_main!(benches);
