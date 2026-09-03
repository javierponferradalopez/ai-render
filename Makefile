.PHONY: verify fmt lint test build

verify: fmt lint test

fmt:
	cargo fmt --check

lint:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

build:
	cargo build --release
