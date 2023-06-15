# quill dissolve-delay

Signs a neuron configuration change to increase a neuron's dissolve delay.

## Basic usage

The basic syntax for running `quill dissolve-delay` commands is:

```bash
quill dissolve-delay <NEURON_ID> <--increase-by <DURATION>|--increase-to <DURATION>>
```

## Arguments

| Argument      | Description                       |
|---------------|-----------------------------------|
| `<NEURON_ID>` | The ID of the neuron to configure |

## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option                     | Description                                            |
|----------------------------|--------------------------------------------------------|
| `--increase-by <DURATION>` | Additional time to add to the neuron's dissolve delay. |
| `--increase-to <DURATION>` | Total time to set the neuron's dissolve delay to.      |

## Examples

The `quill dissolve-delay` command is used to increase a neuron's dissolve delay.

For example, to set it to four years:

```sh
quill dissolve-delay $neuron --increase-to 4y
```

Or to add an additional six months:

```sh
quill dissolve-delay $your_neuron --increase-by 6months
```

This will return a response like:

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

For technical reasons, to hit the eight-year maximum you will need to use `--increase-by`, not `--increase-to`.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro