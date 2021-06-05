.PHONY: all build check clippy test

all: check build clippy test fmt

build:
	cargo build

check:
	cargo check --all --all-targets --all-features --tests

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy --all --all-targets --all-features --tests -- -D warnings

test:
	cd tests && ./run.sh
