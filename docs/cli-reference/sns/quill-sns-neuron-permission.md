# quill sns neuron-permission

Signs a ManageNeuron message to add or remove permissions for a principal to/from a neuron.

This will selectively enable/disable that principal to do a variety of management tasks for the
neuron, including voting and disbursing.

## Basic usage

The basic syntax for running `quill sns neuron-permission` commands is:

```bash
quill sns neuron-permission <NEURON_ID> <SUBCOMMAND> --principal <PRINCIPAL> --permissions <PERMISSIONS>... 
```

## Arguments

| Argument       | Description                                            |
|----------------|--------------------------------------------------------|
| `<SUBCOMMAND>` | Whether to add or remove permissions. (`add`/`remove`) |
| `<NEURON_ID>`  | The ID of the neuron to configure.                     |

## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option                        | Description                                          |
|-------------------------------|------------------------------------------------------|
| `--permissions <PERMISSIONS>` | The permissions to add to/remove from the principal. |
| `--principal <PRINCIPAL>`     | The principal to change the permissions of.          |

## Remarks

Multiple permissions can be specified in one command. The possible permissions are:

* `unspecified`
* `configure-dissolve-state`
* `manage-principals`
* `submit-proposal`
* `vote`
* `disburse`
* `split`
* `merge-maturity`
* `disburse-maturity`
* `stake-maturity`
* `manage-voting-permission`

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
