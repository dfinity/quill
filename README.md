# nano

Minimalistic ledger and governance toolkit for cold wallets.

## Usage

This will save a transfer transaction into the specified file:

    nano nano ledger transfer <account-id> --amount <amount> --memo <memo> --file <path>

`nano` (or `dfx`) could be used on an online computer to send the file:

    nano send <path-to-file>

## Roadmap

0. ~~Support for offline signing of transfer transactions.~~
1. Human readable pretty print of the signed transcation. (Help needed!)
2. Support for governance (neuron staking, dissolving, etc.)
3. Reduce the code and dependnecies to the bare minimum for easy auditing.
4. Get rid of nix.

## Building

0. `nix-shell`
1. `cargo build --release`
