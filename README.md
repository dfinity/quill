# quill

Minimalistic ledger and governance toolkit for cold wallets.

## Disclaimer

YOU EXPRESSLY ACKNOWLEDGE AND AGREE THAT USE OF THIS SOFTWARE IS AT YOUR SOLE RISK.
AUTHORS OF THIS SOFTWARE SHALL NOT BE LIABLE FOR DAMAGES OF ANY TYPE, WHETHER DIRECT OR INDIRECT.

## Usage

This will sign a transfer transaction and print to STDOUT:

    quill --pem-file <path> transfer <account-id> --amount <amount>

To display the signed message in human-readable form:

    quill send --dry-run <path-to-file>

`quill` could be used on an online computer to send any signed transaction:

    quill send <path-to-file>

To get the principal and the account id:

    quill --pem-file <path> public-ids

### Governance

This is how you’d stake/topup a neuron:

    quill --pem-file <path> neuron-stake --amount 2.5 --name 1

Managing the neuron:

    quill --pem-file <path> neuron-manage <neuron-id> [OPERATIONS]

All of the commands above will generate signed messages, which can be sent on the online machine using the `send` command from above.

## Download

Use binaries from the latest [release](https://github.com/dfinity/quill/releases).

## Build

To compile `quill` run:

1. `rustup set profile minimal`
2. `rustup toolchain install stable --component rustfmt --component clippy`
3. `rustup override set stable`
4. `make release`

After this, find the binary at `target/release/quill`.

### Building with Nix

If you have Nix installed, you can use it to provide an environment for
running `cargo`. Just replace the above build steps with the following:

To compile `quill` run:

1. `nix-shell`
4. `make release`

After this, find the binary at `target/release/quill`.

## Testnets

If you have access to an Internet Computer testnet (for example, a version the
replica binary and NNS running locally), you can target quill at this test
network by setting the `IC_URL` environment variable to the full URL. For
example:

    IC_URL=https://nnsdapp.dfinity.network quill --pem-file <path> list-neurons

## Contribution

`quill` is a very critical link in the workflow of the management of valuable assets.
`quill`'s code must stay clean, simple, readable and leave no room for ambiguities, so that it can be reviewed and audited by anyone.
Hence, if you would like to propose a change, please adhere to the following principles:

1. Be concise and only add functional code.
2. Optimize for correctness, then for readability.
3. Avoid adding dependencies at all costs unless it's completely unreasonable to do so.
4. Every new feature (+ a test) is proposed only after it was tested on real wallets.
5. Increment the last digit of the crate version whenever the functionality scope changes. 

## Credit

Originally forked from the [SDK](https://github.com/dfinity/sdk).
