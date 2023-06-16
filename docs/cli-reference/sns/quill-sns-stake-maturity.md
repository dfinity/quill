# quill sns stake-maturity

Signs a ManageNeuron message to stake a percentage of a neuron's maturity.


## Basic usage

The basic syntax for running `quill sns stake-maturity` commands is:

```bash
quill sns stake-maturity <NEURON_ID> <--percentage <PERCENTAGE>|--automatic|--disable-automatic> [option]
```

## Arguments

| Argument      | Description                        |
|---------------|------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to configure. |

## Flags

| Flag                  | Description                         |
|-----------------------|-------------------------------------|
| `-h`, `--help`        | Displays usage information.         |
| `--automatic`         | Enable automatic maturity staking.  |
| `--disable-automatic` | Disable automatic maturity staking. |

## Options

| Option                      | Description                                             |
|-----------------------------|---------------------------------------------------------|
| `--percentage <PERCENTAGE>` | The percentage of the current maturity to stake (1-100) |

## Remarks

A neuron's total stake is the combination of its staked governance tokens and staked maturity.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
