# quill sns status

Fetches the status of the canisters in the SNS. This includes their controller, running status, canister settings, cycle balance, memory size, daily cycle burn rate, and module hash, along with their principals.

## Basic usage

The basic syntax for running `quill sns status` commands is:

```bash
quill sns status [option]
```

## Flags

| Flag           | Description                                        |
|----------------|----------------------------------------------------|
| `--dry-run`    | Will display the query, but not send it.           |
| `-h`, `--help` | Displays usage information.                        |
| `--yes`        | Skips confirmation and sends the message directly. |
