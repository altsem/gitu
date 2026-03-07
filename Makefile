.PHONY: *

test:
	cargo insta test --unreferenced reject
	cargo bench --no-run
	cargo clippy -- -Dwarnings
	cargo fmt --check
  # FIXME cargo deny complaining about vulnerabilities
	# cargo deny check
	git cliff --unreleased

flamegraph:
	cargo flamegraph --profile profiling --bin gitu

heaptrack:
	cargo build --profile profiling
	heaptrack target/profiling/gitu

test-depend:
	cargo install cargo-insta git-cliff || true

