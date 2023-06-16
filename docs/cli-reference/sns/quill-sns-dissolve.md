# quill sns dissolve

Signs a neuron configuration message to start or stop dissolving.

## Basic usage

The basic syntax for running `quill sns dissolve` commands is:

```bash
quill sns dissolve <NEURON_ID> <--start|--stop>
```

## Arguments

| Argument      | Description                |
|---------------|----------------------------|
| `<NEURON_ID>` | The neuron being dissolved |


## Flags

| Flag           | Description                                   |
|----------------|-----------------------------------------------|
| `-h`, `--help` | Displays usage information.                   |
| `--start`      | The neuron will go into the dissolving state. |
| `--stop`       | The neuron will exit the dissolving state.    |

## Remarks

When the neuron goes into the dissolving state, a countdown timer will begin. When the timer is exhausted (i.e. the dissolve delay has elapsed), the neuron can be disbursed. When the neuron exits the dissolving state, whatever amount of time is left in the countdown timer is stored. A neuron's dissolve delay can be extended (for instance to increase voting power) by using the [`quill sns dissolve-delay`] command.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

[`quill sns dissolve-delay`]: quill-sns-dissolve-delay.md
