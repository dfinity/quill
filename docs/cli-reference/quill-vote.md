# quill vote

Signs a neuron management message to vote on a proposal.

## Basic usage

The basic syntax for running `quill vote` commands is:

```bash
quill vote <NEURON_ID> --proposal-id <PROPOSAL_ID> <--approve|--reject>
```

## Arguments

| Argument      | Description                        |
|---------------|------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to vote with. |


## Flags

| Flag           | Description                   |
|----------------|-------------------------------|
| `-h`, `--help` | Displays usage information.   |
| `--approve`    | Vote to approve the proposal. |
| `--reject`     | Vote to reject the proposal.  |

## Options

| Option                        | Description                        |
|-------------------------------|------------------------------------|
| `--proposal-id <PROPOSAL_ID>` | The ID of the proposal to vote on. |

## Examples

The `quill vote` command is used to vote on NNS proposals.

To approve, for example, proposal [108005]:

```sh
quill vote $neuron --proposal-id 108005 --approve
```

Or to reject that same proposal:

```sh
quill vote $neuron --proposal-id 108005 --reject
```

This will produce a response like:

```candid
(
    record {
        command = opt variant {
            RegisterVote = record {}
        };
    }
)
```

## Remarks

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro
[108005]: https://dashboard.internetcomputer.org/proposal/108005
