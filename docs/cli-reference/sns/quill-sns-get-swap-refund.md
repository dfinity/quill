# quill sns get-swap-refund

Signs a message to request a refund from the SNS swap canister.

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

## Examples

The `quill sns get-swap-refund` command is used to refund ICP from an initial token sale (minus transaction fees) if the sale was aborted or failed, or some of your contributed ICP never made it into a neuron:

```sh
quill sns get-swap-refund
```

This will return a response like:

```candid
(
    record {
        result = variant {
            Ok = record {
                block_height = 5_581_035 : nat64;
            }
        };
    }
)
```

The provided block index, also known as block height, can be looked up on the [IC dashboard].

## Remarks

If no principal is provided, the sender's principal will be used.

As this is an update call, it will not actually make the request, but rather generate a signed and packaged request that can be sent from anywhere. You can use the `--qr` flag to display it as a QR code, or if you are not working with an air-gapped machine, you can pipe it to `quill send`.

[IC dashboard]: https://dashboard.internetcomputer.org/transactions
