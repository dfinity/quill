# quill disburse

Signs a disbursal message to convert a dissolved neuron into ICP.

## Basic usage

The basic syntax for running `quill disburse` commands is:

```bash
quill disburse <NEURON_ID> [option]
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

| Option                      | Description                         |
|-----------------------------|-------------------------------------|
| `--amount <AMOUNT>`         | The number of tokens to disburse.   |
| `--subaccount <SUBACCOUNT>` | The subaccount to transfer to.      |
| `--to <TO>`                 | The account to transfer the ICP to. |

## Examples

The `quill disburse` command liquidates a fully dissolved neuron into ICP.

The simplest case is to disburse the whole neuron to yourself:

```sh
quill disburse $neuron
```

Or to, for example, disburse 2.5 ICP to the anonymous principal `2vxsx-fae`:

```sh
quill disburse $neuron --to 2vxsx-fae --amount 2.5
```

This will produce a response like:

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

This block height, also known as the block index, can be looked up on the [IC dashboard].

## Remarks

If `--to` is unset, it will default to the caller; if `--amount` is unset, it will fully consume the neuron.

Only a fully dissolved neuron can be disbursed. To start dissolving a neuron, see [`quill dissolve`].

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro
[`quill dissolve`]: quill-dissolve.md
[IC dashboard]: https://dashboard.internetcomputer.org/transactions
