# quill sns split

Splits a neuron into two neurons.

## Basic usage

The basic syntax for running `quill sns split` commands is:

```bash
quill sns split <NEURON_ID> --amount <AMOUNT> <--name <NAME>|--nonce <NONCE>> [option]
```

## Arguments

| Argument      | Description          |
|---------------|----------------------|
| `<NEURON_ID>` | The neuron to split. |


## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option              | Description                                          |
|---------------------|------------------------------------------------------|
| `--nonce <NONCE>`   | A number to identify this neuron.                    |
| `--name <NAME>`     | A name to identify this neuron.                      |
| `--amount <AMOUNT>` | The number of tokens, in decimal form, to split off. |

## Remarks

As with other commands staking a new neuron, `<NONCE>`/`<NAME>` must be unique among your neurons for this SNS.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
