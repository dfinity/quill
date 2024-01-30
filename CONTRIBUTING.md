# Contributing to `quill`

Thank you for your interest in contributing to `quill`! By participating in this project, you agree to abide by our [Code of Conduct](https://github.com/dfinity/ic-docutrack/blob/main/.github/CODE_OF_CONDUCT.md)

## CLA

All code contributions are subject to our [Contributor License Agreement (CLA)](https://github.com/dfinity/cla/blob/master/CLA.md). When you open your first PR, the CLA bot will prompt you to agree to the CLA before the PR can be reviewed and merged.

## Guidelines

`quill` is a very critical link in the workflow of the management of valuable assets. `quill`'s code must stay clean, simple, readable and leave no room for ambiguities, so that it can be reviewed and audited by anyone. Hence, if you would like to propose a change, please adhere to the following principles:

1. Be concise and only add functional code.
2. Optimize for correctness, then for readability.
3. Avoid adding dependencies at all costs unless it's completely unreasonable to do so.
4. Every new feature (+ a test) is proposed only after it was tested on real wallets.

## Tests

Quill has three kinds of tests to be aware of: 

- Rust unit tests. Add these whenever internal helpers are added or changed, using standard Rust test conventions.
- Output tests. These are located in `tests/output`. Add these whenever the command interface is updated. Each test is an integration test dry-running a `quill` command; its output is checked against a file in `default`. Set `FIX_OUTPUTS=1` when running `cargo test` to generate this file for a new test. Ensure that the output contains Candid field names rather than field hashes.
    - Some output tests require a Ledger device. To run these tests, a Ledger Nano should be plugged in and initialized with the seed "equip will roof matter pink blind book anxiety banner elbow sun young", and the ICP app must be installed. Then run `cargo test -- --ignored` and accept each request on the device. These tests are manually run so if you do not own a Ledger device you will need to wait for the reviewer to notify you of test failures.
- End-to-end tests. These are located in `e2e/tests-quill`. These should be added whenever a test needs to be run against a real replica. They are run with [`bats`](https://github.com/bats-core/bats-core), and you will need to install [`dfx`](https://github.com/dfinity/sdk) and [`bats-support`](https://github.com/ztombol/bats-support), and point the `BATSLIB` environment variable to the `bats-support` installation path. More examples of writing tests for our E2E test setup can be found in the [SDK repository](https://github.com/dfinity/sdk).

## Documentation

Every change to the command-line interface must contain documentation; we use `clap`, so Rustdoc comments turn into CLI documentation. Additionally, this in-code documentation must be mirrored by a corresponding change in `docs/cli-reference`. See existing doc pages for examples. Finally, any feature or notable bugfix should be mentioned in [CHANGELOG.md](CHANGELOG.md), under the `## Unreleased` header.

## Miscellaneous

Quill employs optional Cargo features for different platforms. Ensure your contribution builds (and lints) on all configurations - this can be automated with the [`cargo-hack`](https://github.com/taiki-e/cargo-hack) tool, as `cargo hack clippy --feature-powerset`.
