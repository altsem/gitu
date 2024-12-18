test:
	cargo test
	cargo bench --no-run
	cargo clippy -- -Dwarnings
	cargo fmt --check
  # FIXME cargo deny complaining about vulnerabilities
	# cargo deny check
	git cliff --unreleased
