# nano

Minimalistic ledger and governance toolkit for cold wallets.

## Disclaimer

YOU EXPRESSLY ACKNOWLEDGE AND AGREE THAT USE OF THIS SOFTWARE IS AT YOUR SOLE RISK.
AUTHORS OF THIS SOFTWARE SHALL NOT BE LIABLE FOR DAMAGES OF ANY TYPE, WHETHER DIRECT OR INDIRECT.

## Usage

This will sign a transfer transaction and print to STDOUT:

    nano --pem-file <path> transfer <account-id> --amount <amount>

To display the signed message in human-readable form:

    nano send --dry-run <path-to-file>

`nano` could be used on an online computer to send the file:

    nano send <path-to-file>

Check the request status:

    nano --pem-file <path> request-status <request-id>

To get the principal id:

    nano --pem-file <path> principal-id

For the account id:

    nano --pem-file <path> account-id

## Roadmap

0. ~~Support for offline signing of transfer transactions.~~
1. ~~Human readable pretty print of the signed transcation. (Help needed!)~~
2. Support for governance (neuron staking, dissolving, etc.)
3. Reduce the code and dependnecies to the bare minimum for easy auditing.
4. ~~Get rid of nix.~~

## Building

0. Build by running `cargo build --release`.
1. Find the binary at `target/release/nano`.

## Source Code

Derived from [SDK](https://github.com/dfinity/sdk).
