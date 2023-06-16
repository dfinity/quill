# quill sns get-sale-participation

Queries for how much ICP a user has contributed to a token sale.

## Basic usage

The basic syntax for `quill sns get-sale-participation` commands is:

```sh
quill sns get-sale-participation [option]
```

## Flags

| Flag           | Description                                        |
|----------------|----------------------------------------------------|
| `--dry-run`    | Will display the query, but not send it.           |
| `-h`, `--help` | Displays usage information.                        |
| `--yes`        | Skips confirmation and sends the message directly. |

## Options

| Option                    | Description             |
|---------------------------|-------------------------|
| `--principal <PRINCIPAL>` | The principal to query. |

## Examples

The `quill sns get-sale-participation` command is used to check how much ICP you or another user have spent on the initial token sale.

To check your own:

```sh
quill sns get-sale-participation
```

This will return a response like:

```candid
(
    record {
        buyer_state = opt record {
            icp = opt record {
                transfer_fee_paid_e8s = opt 10_000 : nat64;
                transfer_start_timestamp_seconds = 1_669_073_904 : nat64;
                amount_e8s = 1_000_000_000 : nat64;
                amount_transferred_e8s = opt 350_000_000 : nat64;
                transfer_success_timestamp_seconds = 1_669_075_032 : nat64;
            }
        };
    }
)
```

The timestamps are in Unix time, or seconds since midnight on January 1, 1970; the start date here was on 2022-11-21. The ICP amounts are in e8s, or hundred-millionths of an ICP; the amount transferred here is 3.5 ICP.

## Remarks

If the principal is unspecified, the caller's principal will be used.

As this is a query call, it cannot be executed on an air-gapped machine, but does not require access to your keys.
