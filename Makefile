.PHONY: all build check clippy test

all: check clippy fmt build test

build:
	cargo build

release:
	cargo build --release --locked

musl-static:
	cargo build --target x86_64-unknown-linux-musl --release --locked

check:
	cargo check --all --all-targets --all-features --tests

fmt:
	cargo fmt -- --check

clippy:
	cargo clippy --all --all-targets --all-features --tests -- -D warnings

test:
	cargo build
	cd tests && ./run.sh
