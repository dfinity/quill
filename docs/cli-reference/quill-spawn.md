# quill spawn

Signs a neuron management message to convert a neuron's maturity into a rapidly-dissolving neuron.

## Basic usage

The basic syntax for running `quill spawn` commands is:

```bash
quill spawn <NEURON_ID> [option]
```

## Arguments

| Argument      | Description                         |
|---------------|-------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to spawn from. |


## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option                      | Description                              |
|-----------------------------|------------------------------------------|
| `--percentage <PERCENTAGE>` | The percentage of the maturity to spawn. |
| `--to <TO>`                 | The owner of the spawned neuron.         |

## Examples

The `quill spawn` command is used to convert neuron maturity into ICP by splitting it off into a neuron.

The simplest case is to spawn all of the maturity:

```sh
quill spawn $neuron
```

Or to, for example, spawn 25% of the maturity to a different principal:

```sh
quill spawn $neuron --percentage 25 --to "$(dfx identity get-principal)"
```

This will return a response like:

```candid
(
    record {
        command = opt variant {
            Spawn = record {
                created_neuron_id = 2_313_380_519_530_470_538 : nat64;
            }
        };
    }
)
```

## Remarks

Spawned neurons will immediately start dissolving, and take seven days to dissolve. When the neuron is done dissolving, use [`quill disburse`] to liquidate it into ICP. Spawning maturity just to re-stake it is less efficent than staking it directly; to do that, see [`quill stake-maturity`].

If `--to` is unset, it will default to the caller; if `--percentage` is unset it will default to 100%.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

For more information about neurons, see [Neurons]; for more information about their role in the NNS, see [Network Nervous System][NNS].

[Neurons]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-staking-voting-rewards#neurons
[NNS]: https://internetcomputer.org/docs/current/tokenomics/nns/nns-intro
[`quill disburse`]: quill-disburse.md
[`quill stake-maturity`]: quill-stake-maturity.md
