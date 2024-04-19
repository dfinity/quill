# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

- Overhauled output format. All commands besides `quill sns` should have human-readable output instead of candid IDL. Candid IDL format can be forced with `--raw`.

## [0.4.4] - 2024-03-21

- Fixed `quill sns make-proposal` setting some fields to null.

## [0.4.3] - 2024-01-29

- Updated dependencies.

## [0.4.2] - 2023-06-21

- Added `--subaccount` to `quill public-ids`. (#201)
- Added Ledger support via `--ledger`. (#199)
- Added `--confirmation-text` to `quill sns pay`. (#195)
- Fixed `quill ckbtc update-balance` allowing the anonymous principal. (#191)
- Added `disburse`, `disburse-maturity`, `split-neuron`, and `follow-neuron` to `quill sns`. (#191)
- Added option to print DFN address for Genesis investors. (#184)
- Updated to new ICRC-1 account ID format. (#190)

## [0.4.1] - 2023-03-23

- Added release binaries for linux-gnu in addition to linux-musl on amd64. (#180)
- Fixed `quill generate` requiring authentication. (#181)
- Require an additional `--already-transferred` flag for the single-message form of `quill neuron-stake`. (#173)
- Added `--disburse-amount` and `--disburse-to` to `quill neuron-manage`. (#171)
- Accepts bare principals and ICRC-1 account IDs in `quill account-balance` and `quill transfer`. (#168)
- Allowed omitting the account ID in `quill account-balance`. (#167)
- Added `--from-subaccount` to `quill transfer` and `quill neuron-stake`. (#166)
- Added `--summary-path` to `quill sns make-upgrade-canister-proposal`. (#164)

## [0.4.0] - 2023-02-14

## Changed

- Require `--ticket-creation-time` in `quill sns pay`. (#159)
- `--proposal-path` in the `sns make-proposal` command expects the binary encoding
  of the proposal candid record. (#160)

## [0.3.3] - 2023-02-09

### Changed

- Remove the `EC PARAMETERS` section in the PEM file to match dfx. (#152)

### Added

- ckBTC commands and support. (#153)
- SNS commands and support (replaces sns-quill). (#154)
- Support the new sns sale payment flow for the ticketing system. (#156)

## [0.3.2] - 2023-01-13

### Changed
- Bump `openssl` crate to 0.10.45

## [0.3.1] - 2022-12-20

### Changed
- Added auto-staking maturity. (#141)
- Removed the ability to merge maturity, replaced with staking maturity. (#140)
- Removed range voting. (#139)

## [0.3.0] - 2022-10-11

### Added
- `neuron-manage register-vote`. (#132)
-  Range voting. (#136)
### Changed
- All command parameters have been moved to the end of the command. (#126)

### Fixed
- `quill generate` arg. (#131)

## [0.2.17] - 2022-07-13

### Fixed
- The generated PEM file now have correct `EC PARAMETERS` of secp256k1. (#124)

## [0.2.16] - 2022-06-22

### Added
- New command `replace-node-provider-id`. (#118)

### Changed
- All queries are performed as update equivalents, in order to certify their responses. (#115)
