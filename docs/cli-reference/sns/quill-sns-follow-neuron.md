# quill sns follow-neuron

Configures a neuron to follow another neuron or group of neurons.

## Basic usage

The basic syntax for running `quill sns follow-neuron` commands is:

```bash
quill sns follow-neuron <NEURON_ID> <--type <TYPE>|--function-id <FUNCTION_ID>> <--followees <FOLLOWEES>|--unfollow> [option]
```

## Arguments

| Argument      | Description              |
|---------------|--------------------------|
| `<NEURON_ID>` | The neuron to configure. |

## Flags

| Flag           | Description                                      |
|----------------|--------------------------------------------------|
| `-h`, `--help` | Displays usage information.                      |
| `--unfollow`   | Remove any followees for this proposal function. |

## Options

| Option                        | Description                                                         |
|-------------------------------|---------------------------------------------------------------------|
| `--followees <FOLLOWEES>`     | A list of neurons to follow for this proposal function              |
| `--function-id <FUNCTION_ID>` | The numeric ID of the proposal function to restrict following to    |
| `--type <TYPE>`               | The name of the built-in proposal function to restrict following to |

## Examples

The `quill sns follow-neuron` command is used to follow other neurons. Following a neuron causes your neuron to automatically vote when that neuron does. 

To follow another neuron on all topics:

```sh
quill sns follow-neuron $your_neuron --followees $their_neuron --type all
```

You can also follow a group of neurons, and vote when the majority (by voting power) of the followed neurons do:

```sh
quill sns follow-neuron $your_neuron --followees $one_neuron,$another_neuron --type all
```

Follow relationships are granular by the purpose of a proposal. To follow a neuron only for votes on transferring SNS treasury funds:

```sh
quill sns follow-neuron $your_neuron --followees $their_neuron --type transfer-sns-treasury-funds
```

There are several built-in proposal functions which this command accepts by name, but new ones can be added at any time via AddGenericNervousSystemFunction proposals and those must be addressed by integer ID:

```sh
quill sns follow-neuron $your_neuron --followees $their_neuron --function-id 257
```

You can also clear your followed neurons for a particular topic. To return to manual voting on e.g. motion proposals:

```sh
quill sns follow-neuron $your_neuron --unfollow --type motion
```

These will return responses like:

```candid
(
    record {
        command = opt variant {
            Follow = record {}
        };
    }
)
```

## Remarks

Built-in proposal type names: `all`, `motion`, `manage-nervous-system-parameters`, `upgrade-sns-controlled-canister`, `add-generic-nervous-system-function`,
`remove-generic-nervous-system-function`, `upgrade-sns-to-next-version`, `manage-sns-metadata`, `transfer-sns-treasury-funds`, `register-dapp-canisters`,`deregister-dapp-canisters`.

Any follow command overrides a previous follow command for the same topic; the followee list will be replaced, not added to.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
