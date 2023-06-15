# quill split

Signs a neuron management message to split a neuron in two.

## Basic usage

The basic syntax for running `quill split` commands is:

```bash
quill split <NEURON_ID> --amount <AMOUNT>
```

## Arguments

| Argument      | Description                    |
|---------------|--------------------------------|
| `<NEURON_ID>` | The ID of the neuron to split. |


## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option              | Description                                          |
|---------------------|------------------------------------------------------|
| `--amount <AMOUNT>` | The amount of the stake that should be split, in ICP |

## Examples

The `quill split` command is used to split a neuron into two neurons.

To split, for example, 10 ICP off into a new neuron:

```sh
quill split $neuron --amount 10
```

This will return a response like:

```candid
(
    record {
        command = opt variant {
            Spawn = record {
                created_neuron_id = 2_313_380_519_530_470_538 : nat64;
            }
        };
    }
)
```

## Remarks

Splitting a neuron has a few use-cases, but the most common one is to dissolve only part of a stake without sacrificing the accrued age bonus.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro
