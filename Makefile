test:
	cargo test
	cargo bench --no-run
	cargo clippy -- -Dwarnings
	cargo fmt --check
	cargo deny check
	git cliff --unreleased
