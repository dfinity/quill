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

To get the principal and the account id:

    nano --pem-file <path> public-ids

### Governance

This is how you’d stake/topup a neuron:

    nano --pem-file <path> neuron-stake --amount 2.5 --name 1

Managing the neuron:

    nano --pem-file <path> neuron-manage <neuron-id> [OPERATIONS]

Currently supported operations are: `--start-dissolving`, `--stop-dissolving`, `--disburse`, `--add-hot-key`, `--remove-hot-key`.

All of the commands above will generate signed messages, which can be sent on the online machine using the `send` command from above.

## Credit

Originally forked from the [SDK](https://github.com/dfinity/sdk).
