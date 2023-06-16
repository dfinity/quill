# quill sns list-deployed-snses

Lists all SNSes that have been deployed by the NNS.

## Basic usage

The basic syntax for running `quill sns list-deploy-snses` commands is:

```bash
quill sns list-deployed-snses [option]
```

## Flags

| Flag           | Description                                        |
|----------------|----------------------------------------------------|
| `--dry-run`    | Will display the query, but not send it.           |
| `-h`, `--help` | Displays usage information.                        |
| `-y`, `--yes`  | Skips confirmation and sends the message directly. |

## Examples

The `quill sns list-deployed-snses` command is used to list all the SNSes that have been deployed by the NNS:

```sh
quill sns list-deployed-snses
```

This will return a response like:

```candid
(
    record {
        instances = vec {
            record {
                root_canister_id = opt principal "zxeu2-7aaaa-aaaaq-aaafa-cai";
                governance_canister_id = opt principal "zqfso-syaaa-aaaaq-aaafq-cai";
                index_canister_id = opt principal "zlaol-iaaaa-aaaaq-aaaha-cai";
                swap_canister_id = opt principal "zcdfx-6iaaa-aaaaq-aaagq-cai";
                ledger_canister_id = opt principal "zfcdd-tqaaa-aaaaq-aaaga-cai";
            };
            record {
                root_canister_id = opt principal "3e3x2-xyaaa-aaaaq-aaalq-cai";
                governance_canister_id = opt principal "2jvtu-yqaaa-aaaaq-aaama-cai";
                index_canister_id = opt principal "2awyi-oyaaa-aaaaq-aaanq-cai";
                swap_canister_id = opt principal "2hx64-daaaa-aaaaq-aaana-cai";
                ledger_canister_id = opt principal "2ouva-viaaa-aaaaq-aaamq-cai";
            };
        }
    }
)
```

## Remarks

As this is a query call, it cannot be executed on an air-gapped machine, but does not require access to your keys.
