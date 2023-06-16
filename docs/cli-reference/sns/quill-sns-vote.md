# quill sns vote

Signs a ManageNeuron message to register a vote for a proposal.

## Basic usage

The basic syntax for running `quill sns vote` commands is:

```bash
quill sns vote <NEURON_ID> --proposal-id <PROPOSAL_ID> <--approve|--reject> [option]
```

## Arguments

| Argument      | Description                        |
|---------------|------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to configure. |

## Flags

| Flag           | Description                   |
|----------------|-------------------------------|
| `-h`, `--help` | Displays usage information.   |
| `--approve`    | Vote to approve the proposal. |
| `--reject`     | Vote to reject the proposal.  |

## Options

| Option                        | Description                            |
|-------------------------------|----------------------------------------|
| `--proposal-id <PROPOSAL_ID>` | The ID of the proposal to be voted on. |

## Remarks

Registering a vote will update the ballot of the given proposal and could trigger followees to vote. When enough votes are cast or enough time passes, the proposal will either be rejected or adopted and executed.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
