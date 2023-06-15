# quill stake-maturity

Signs a neuron management message to add maturity to a neuron's stake, or configure auto-staking.

## Basic usage

The basic syntax for running `quill stake-maturity` commands is:

```bash
quill stake-maturity <NEURON_ID> <--percentage <PERCENTAGE>|--automatic|--disable-automatic>>
```

## Arguments

| Argument      | Description                     |
|---------------|---------------------------------|
| `<NEURON_ID>` | The ID of the neuron to manage. |


## Flags

| Flag                  | Description                         |
|-----------------------|-------------------------------------|
| `-h`, `--help`        | Displays usage information.         |
| `--automatic`         | Enable automatic maturity staking.  |
| `--disable-automatic` | Disable automatic maturity staking. |

## Options

| Option                      | Description                                               |
|-----------------------------|-----------------------------------------------------------|
| `--percentage <PERCENTAGE>` | The percentage of the neuron's accrued maturity to stake. |

## Examples

The `quill stake-maturity command is used to stake a portion of the neuron's accrued maturity.

To stake 25% of the neuron's maturity:

```sh
quill stake-maturity $neuron --percentage 25
```

This will produce a response like:

```candid
(
    record {
        commmand = opt variant {
            StakeMaturity = record {
                maturity_e8s = 450_000_000 : nat64;
                staked_maturity_e8s = 150_000_000 : nat64;
            }
        }
    }
)
```

The provided numbers are in e8s, or hundred-millionths of an ICP. In this case 1.5 ICP worth of maturity was staked, out of an initial quantity of 6.0.

You can also configure the neuron to automatically stake all maturity:

```sh
quill stake-maturity $neuron --automatic
```

Or, if this was enabled previously, to disable it:

```sh
quill stake-maturity $neuron --disable-automatic
```

This will produce a response like:

```candid
(
    record {
        command = opt variant {
            Configure = record {}
        };
    }
)
```

## Remarks

A neuron's total voting power is proportional to the sum of its staked ICP and staked maturity.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro
