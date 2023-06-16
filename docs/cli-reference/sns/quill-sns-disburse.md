# quill sns disburse

Converts a fully-dissolved neuron into SNS utility tokens.

## Basic usage

The basic syntax for running `quill sns disburse` commands is:

```bash
quill sns disburse <NEURON_ID> [option]
```

## Arguments

| Argument      | Description             |
|---------------|-------------------------|
| `<NEURON_ID>` | The neuron to disburse. |

## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option                      | Description                                           |
|-----------------------------|-------------------------------------------------------|
| `--to <TO>`                 | The account to transfer the SNS utility tokens to.    |
| `--subaccount <SUBACCOUNT>` | The subaccount to transfer the SNS utility tokens to. |
| `--amount <AMOUNT>`         | The number of tokens, in decimal form, to disburse.   |

## Examples

The `quill sns disburse` command is used to liquidate a neuron into its staked tokens, after it has been dissolved.

The simplest case is to disburse all the tokens to yourself:

```sh
quill sns disburse $token
```

But to disburse, for example, 5 tokens to the anonymous principal `2vxsx-fae`:

```sh
quill sns disburse $token --to 2vxsx-fae --amount 5
```

This will return a response like:

```candid
(
    record {
        command = opt variant {
            Disburse = record {
                transfer_block_height = 5_581_035 : nat64;
            }
        };
    }
)
```

The provided block index, also known as the block height, can be looked up on the [IC dashboard].

## Remarks

Only a neuron that has fully dissolved may be disbursed. To start dissolving a neuron, see [`quill sns configure-dissolve-delay`].

If `<TO>` is unset, it will default to the caller; if `<AMOUNT>` is unset, it will fully consume the neuron.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

[`quill sns configure-dissolve-delay`]: quill-sns-configure-dissolve-delay.md
[IC dashboard]: https://dashboard.internetcomputer.org/sns
