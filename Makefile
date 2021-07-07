.PHONY: all build check clippy test

all: check clippy fmt build test

build:
	cargo build

release:
	cargo build --release --locked

check:
	cargo check --all --all-targets --all-features --tests

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy --all --all-targets --all-features --tests -- -D warnings

test:
	cargo build
	cd tests && ./run.sh
