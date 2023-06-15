# quill sns get-swap-refund

Signs a message to request a refund from the SNS swap canister. If the swap was aborted or failed, or some of your contributed ICP never made it into a neuron, this command can retrieve your unused ICP, minus transaction fees.

## Basic usage

The basic syntax for running `quill sns get-swap-refund` commands is:

```bash
quill sns get-swap-refund [option]
```

## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option                    | Description                                                          |
|---------------------------|----------------------------------------------------------------------|
| `--principal <PRINCIPAL>` | The principal that made the ICP contribution and should be refunded. |

## Remarks

If no principal is provided, the sender's principal will be used.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.
