use criterion::{criterion_group, criterion_main, Criterion};
use gitu::cli::Commands;
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
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(100)).sample_size(10);
    targets = show
}
criterion_main!(benches);

use std::{fs::File, os::raw::c_int, path::Path};

use criterion::profiler::Profiler;
use pprof::ProfilerGuard;

pub struct FlamegraphProfiler<'a> {
    frequency: c_int,
    active_profiler: Option<ProfilerGuard<'a>>,
}

impl<'a> FlamegraphProfiler<'a> {
    #[allow(dead_code)]
    pub fn new(frequency: c_int) -> Self {
        FlamegraphProfiler {
            frequency,
            active_profiler: None,
        }
    }
}

impl<'a> Profiler for FlamegraphProfiler<'a> {
    fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        std::fs::create_dir_all(benchmark_dir).unwrap();
        let flamegraph_path = benchmark_dir.join("flamegraph.svg");
        let flamegraph_file = File::create(&flamegraph_path)
            .expect("File system error while creating flamegraph.svg");
        if let Some(profiler) = self.active_profiler.take() {
            profiler
                .report()
                .build()
                .unwrap()
                .flamegraph(flamegraph_file)
                .expect("Error writing flamegraph");
        }
    }
}
