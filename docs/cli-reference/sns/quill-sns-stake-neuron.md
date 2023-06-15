# quill sns stake-neuron

Signs messages needed to stake governance tokens for a neuron. First, stake-neuron will sign a ledger transfer to a subaccount of the Governance canister calculated from the provided private key and memo. Second, stake-neuron will sign a ManageNeuron message for Governance to claim the neuron for the principal derived from the provided private key.

## Basic usage

The basic syntax for running `quill sns stake-neuron` commands is:

```bash
quill sns stake-neuron <--name <NAME>|--nonce <NONCE>> --amount <AMOUNT> [option]
```

## Flags

| Flag                    | Description                 |
|-------------------------|-----------------------------|
| `-h`, `--help`          | Displays usage information. |
| `--already-transferred` | No transfer will be made.   |

## Options

| Option                                | Description                                                               |
|---------------------------------------|---------------------------------------------------------------------------|
| `--amount <AMOUNT>`                   | The amount of tokens in e8s to be transferred to the Governance canister. |
| `--fee <FEE>`                         | The amount that the caller pays for the transaction.                      |
| `--from-subaccount <FROM_SUBACCOUNT>` | The subaccount to make the transfer from.                                 |
| `--nonce <NONCE>`                     | An arbitrary number to identify this neuron.                              |
| `--name <NAME>`                       | A name to identify this neuron.                                           |

## Remarks

The amount of specified tokens will be transferred to the governance canister's ledger subaccount (the neuron's account ID) from the account ID derived from the provided private key. This is known as a staking transfer. These funds will be returned when disbursing the neuron. If an amount is _not_ specified, no transfer will be made, and only a neuron claim command will be signed. This is useful for situations where the transfer was initially made with some other command or tool.

The memo must be unique among the neurons claimed for a single principal. More information on ledger accounts and subaccounts can be found here: [Ledger Canister Overview](https://smartcontracts.org/docs/integration/ledger-quick-start.html#_ledger_canister_overview)

The default fee is 0.0001 tokens. Use the `--fee` flag when using an SNS that sets its own transaction fee.

If `--already-transferred` is specified, then only the neuron claim message will be generated. This is useful if there was an error previously submitting the notification which you have since rectified, or if you have made the transfer with another tool.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
