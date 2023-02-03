# quill sns swap

Signs messages needed to participate in the initial token swap. This operation consists of two messages: First, `amount` ICP is transferred to the swap canister on the NNS ledger, under the subaccount for your principal. Second, the swap canister is notified that the transfer has been made.

## Basic usage

The basic syntax for running `quill sns swap` commands is:

```bash
quill sns swap [option]
```

## Flags

| Flag            | Description                 |
|-----------------|-----------------------------|
| `-h`, `--help`  | Displays usage information. |
| `--notify-only` | No transfer will be made.   |

## Options

| Option              | Description                                                                   |
|---------------------|-------------------------------------------------------------------------------|
| `--amount <AMOUNT>` | The amount of ICP to transfer.                                                |
| `--memo <MEMO>`     | An arbitrary number used to identify the NNS block this transfer was made in. |

## Remarks

Once the swap has been finalized, if it was successful, you will receive your neurons automatically. Your neuron's share of the governance tokens at sale finalization will be proportional to your share of the contributed ICP.

If `--notify-only` is specified, only the notification message will be generated. This is useful if there was an error previously submitting the notification which you have since rectified, or if you have made the transfer with another tool.
