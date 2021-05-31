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

`nano` could be used on an online computer to send any signed transactions:

    nano send <path-to-file>

Sign a request status check:

    nano --pem-file <path> request-status-sign <request-id>

Submit the signed status check:

    nano request-status-submit --file <path>

To get the principal and the account id:

    nano --pem-file <path> public-ids

### Governance

This is how you’d stake/topup a neuron:

    nano --pem-file <path> neuron-stake --amount 2.5 --name 1

Managing the neuron:

    nano --pem-file <path> neuron-manage <neuron-id> [OPERATIONS]

Currently supported operations are: `--start-dissolving`, `--stop-dissolving`, `--disburse`, `--add-hot-key`, `--remove-hot-key`.

All of the commands above will generate signed messages, which can be sent on the online machine like this:

    nano send <file>

## Roadmap

0. ~~Support for offline signing of transfer transactions.~~
1. ~~Human readable pretty print of the signed transcation. (Help needed!)~~
2. ~~Support for governance (neuron staking, dissolving, etc.)~~
3. ~~Reduce the code and dependencies to the bare minimum for easy auditing.~~
4. ~~Get rid of nix.~~
5. Full governance support?

## Building

0. Build by running `cargo build --release`.
1. Find the binary at `target/release/nano`.

## Source Code

Derived from [SDK](https://github.com/dfinity/sdk).
