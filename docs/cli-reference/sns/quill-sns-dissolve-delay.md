# quill sns dissolve-delay

Signs a neuron configure message to increase the dissolve delay of a neuron.

## Basic usage

The basic syntax for running `quill sns dissolve-delay` commands is:

```bash
quill sns dissolve-delay <NEURON_ID> <--increase-by <DURATION>|--increase-to <DURATION>>
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

| Option                     | Description                                                      |
|----------------------------|------------------------------------------------------------------|
| `--increase-by <DURATION>` | Additional time to add to the neuron's dissolve delay, e.g. '1y' |
| `--increase-to <DURATION>` | Total time to set the neuron's dissolve delay to, e.g. '4y'      |

## Remarks

The dissolve delay of a neuron determines its voting power, its ability to vote, its ability to make proposals, and other actions it can take (such as disbursing). If the neuron is already dissolving and this command is used, the neuron will stop dissolving and begin aging.

For technical reasons, to hit the maximum dissolve delay, you will need to use `--increase-by`, not `--increase-to`.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
