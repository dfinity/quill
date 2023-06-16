# quill dissolve

Signs a neuron configuration message to start or stop a neuron dissolving.

## Basic usage

The basic syntax for running `quill dissolve` commands is:

```bash
quill dissolve <NEURON_ID> <--start|--stop>
```

## Arguments

| Argument      | Description                       |
|---------------|-----------------------------------|
| `<NEURON_ID>` | The ID of the neuron to dissolve. |

## Flags

| Flag           | Description                  |
|----------------|------------------------------|
| `-h`, `--help` | Displays usage information.  |
| `--start`      | Start dissolving the neuron. |
| `--stop`       | Stop dissolving the neuron.  |

## Examples

The `quill dissolve` command is used to dissolve a neuron:

```sh
quill dissolve $neuron --start
```

Or to stop one from dissolving:

```sh
quill dissolve $neuron --stop
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

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro
