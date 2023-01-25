# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
- remove the `EC PARAMETERS` section in the PEM file to match dfx

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
