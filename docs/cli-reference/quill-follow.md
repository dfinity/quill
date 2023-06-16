# quill follow

Signs a neuron configuration message to change a neuron's follow relationship.

## Basic usage

The basic syntax for running `quill follow` commands is:

```bash
quill follow <NEURON_ID> <--type <TYPE>|--topic-id <TOPIC_ID>> <--followees <FOLLOWEES>|--unfollow>
```

## Arguments

| Argument      | Description                        |
|---------------|------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to configure. |


## Flags

| Flag           | Description                          |
|----------------|--------------------------------------|
| `-h`, `--help` | Displays usage information.          |
| `--unfollow`   | Unfollow all neurons for this topic. |

## Options

| Option                    | Description                                                    |
|---------------------------|----------------------------------------------------------------|
| `--followees <FOLLOWEES>` | A comma-separated list of neuron IDs to follow.                |
| `--topic-id <TOPIC_ID>`   | The numeric ID of the proposal topic to restrict following to. |
| `--type <TYPE>`           | The name of the proposal topic to restrict following to.       |

## Examples

The `quill follow` command is used to follow other neurons. Following a neuron causes your neuron to automatically vote when that neuron does.

To follow another neuron on all topics, for example, ICPMN:

```sh
quill follow $neuron --followees 4966884161088437903 --type all
```

You can also follow a group of neurons, e.g. ICDevs and the ICA, and vote when the majority (by voting power) of the followed neurons do:

```sh
quill follow $neuron --followees 14231996777861930328,28 --type all
```

Follow relationships are granular by the purpose of a proposal. To follow, for example, DFINITY's neuron on exchange rate adjustments:

```sh
quill follow $neuron --followees 27 --type exchange-rate
```

If the name of the topic is not listed in Quill and you know its numeric ID, you can use that; the above command is equivalent to:

```sh
quill follow $neuron --followees 27 --topic-id 2
```

You can also clear your followed neurons for a particular topic. To return to manual voting on e.g. governance proposals:

```sh
quill follow $your-neuron --type governance --unfollow
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

Built-in proposal topic names: `all`, `neuron-management`, `exchange-rate`, `network-economics`, `governance`, `node-admin`, `participant-management`, `subnet-management`, `network-canister-management`, `kyc`, `node-provider-rewards`, `sns-decentralization-sale`, `subnet-replica-version-management`, `replica-version-management`, `sns-and-community-fund`.

Any follow command overrides a previous follow command for the same topic; the followee list will be replaced, not added to.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro
