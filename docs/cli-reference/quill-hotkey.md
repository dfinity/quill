# quill hotkey

Signs a neuron configuration message to add or remove a hotkey.

## Basic usage

The basic syntax for running `quill hotkey` commands is:

```bash
quill hotkey <NEURON_ID> <--add <PRINCIPAL>|--remove <PRINCIPAL>>
```

## Arguments

| Argument      | Description                        |
|---------------|------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to configure. |


## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option                 | Description                                 |
|------------------------|---------------------------------------------|
| `--add <PRINCIPAL>`    | Add the specified principal as a hotkey.    |
| `--remove <PRINCIPAL>` | Remove the specified principal as a hotkey. |

## Examples

The `quill hotkey` command is used to add a principal as a hotkey to a neuron, allowing the principal to vote or follow other neurons for voting without any financial access.

To add, for example, the anonymous principal `2vxsx-fae` as a hotkey:

```sh
quill hotkey $neuron --add 2vxsx-fae
```

Or to remove it:

```sh
quill hotkey $neuron --remove 2vxsx-fae
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