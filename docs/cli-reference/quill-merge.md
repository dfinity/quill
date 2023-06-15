# quill merge

Signs a neuron management message to merge another neuron into this one.

## Basic usage

The basic syntax for running `quill merge` commands is:

```bash
quill merge <NEURON_ID> --from <FROM>
```

## Arguments

| Argument      | Description                         |
|---------------|-------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to merge into. |


## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option          | Description                         |
|-----------------|-------------------------------------|
| `--from <FROM>` | The ID of the neuron to merge from. |

## Examples

The `quill merge` command is used to merge two neurons together, combining their stake, maturity, age, and dissolve delay:

```sh
quill merge $main_neuron --from $other_neuron
```

This will return a response like:

```candid
(
    record {
        command = opt variant {
            Merge = record {}
        };
    }
)
```

## Remarks

Stakes and maturity combine additively. Age combines according to the formula `((age1 * stake1) + (age2 * stake2)) / (stake1 + stake2)`. Dissolve delays combine to whichever neuron's dissolve delay was higher.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro