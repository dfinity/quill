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

## Remarks

If the principal is unspecified, the caller's principal will be used.

As this is a query call, it cannot be executed on an air-gapped machine, but does not require access to your keys.
