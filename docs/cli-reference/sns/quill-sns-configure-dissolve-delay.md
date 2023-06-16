# quill sns configure-dissolve-delay

Signs a ManageNeuron message to configure the dissolve delay of a neuron.

## Basic usage

The basic syntax for running `quill sns configure-dissolve-delay` commands is:

```bash
quill sns configure-dissolve-delay <NEURON_ID> [option]
```

## Arguments

| Argument      | Description                        |
|---------------|------------------------------------|
| `<NEURON_ID>` | The ID of the neuron to configure. |

## Flags

| Flag                 | Description                                   |
|----------------------|-----------------------------------------------|
| `-h`, `--help`       | Displays usage information.                   |
| `--start-dissolving` | The neuron will go into the dissolving state. |
| `--stop-dissolving`  | The neuron will exit the dissolving state.    |

## Options

| Option                                                | Description                                                              |
|-------------------------------------------------------|--------------------------------------------------------------------------|
| `-a`, `--additional-dissolve-delay-seconds <SECONDS>` | Additional number of seconds to add to the dissolve delay of the neuron. |

## Examples

The `quill sns configure-dissolve-delay` command is used to change a neuron's dissolve state.

To start dissolving a neuron:

```sh
quill sns configure-dissolve-delay $neuron --start-dissolving
```

Or to stop:

```sh
quill sns configure-dissolve-delay $neuron --stop-dissolving
```

To increase the dissolve delay by, for example, fifteen seconds:

```sh
quill sns configure-dissolve-delay $neuron --additional-dissolve-delay-seconds 15
```

This will return a response like:

```candid
(
    record {
        command = opt variant {
            Configure = record {};
        };
    }
)
```

## Remarks

The dissolve delay of a neuron determines its voting power, its ability to vote, its ability to make proposals, and other actions it can take (such as disbursing). When the neuron starts dissolving, a countdown timer will begin. When the timer is exhausted (i.e. dissolve_delay_seconds amount of time has elapsed), the neuron can be disbursed. When the neuron stops dissolving, whatever amount of dissolve delay seconds is left in the countdown timer is stored. 

If the neuron is already dissolving and the dissolve delay is increased, the neuron will stop dissolving and begin aging.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
