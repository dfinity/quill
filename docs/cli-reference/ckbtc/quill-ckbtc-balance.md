# quill ckbtc balance

Sends a message to check the provided user's ckBTC balance.

The `--of` parameter is required if a signing key is not provided.

## Basic usage

The basic syntax for running `quill ckbtc balance` commands is:

```bash
quill ckbtc balance [option]
```

## Flags

| Flag           | Description                                        |
|----------------|----------------------------------------------------|
| `--dry-run`    | Will display the query, but not send it.           |
| `-h`, `--help` | Displays usage information.                        |
| `--testnet`    | Uses ckTESTBTC instead of ckBTC.                   |
| `-y`, `--yes`  | Skips confirmation and sends the message directly. |

## Options

| Argument                          | Description                                      |
|-----------------------------------|--------------------------------------------------|
| `--of <OF>`                       | The account to check. Optional if a key is used. |
| `--of-subaccount <OF_SUBACCOUNT>` | The subaccount of the account to check.          |
