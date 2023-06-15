# quill community-fund

Signs a message to join or leave the Internet Computer's community fund with this neuron's maturity.

## Basic usage

The basic syntax for running `quill community-fund` commands is:

```bash
quill community-fund <NEURON_ID> <--join|--leave>
```

## Arguments

| Argument      | Description                        |
|---------------|------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to configure. |

## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |
| `--join`       | Join the community fund     |
| `--leave`      | Leave the community fund    |

## Examples

The `quill community-fund` command is used to make a neuron join or leave the community fund, which allows the NNS to make investments using the neuron's maturity.

To join the community fund:

```sh
quill community-fund $neuron --join
```

Or to leave:

```sh
quill community-fund $neuron --leave
```

These will return a response like:

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

For more information about the community fund, see [Community Fund].

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

[Community Fund]: https://internetcomputer.org/docs/current/tokenomics/nns/community-fund
