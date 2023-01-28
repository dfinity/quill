# quill ckbtc transfer

Signs a message to transfer ckBTC from one account to another.

## Basic usage

The basic syntax for running `quill ckbtc transfer` commands is:

```bash
quill ckbtc transfer [option] <TO>
```

## Arguments

| Argument | Description                       |
|----------|-----------------------------------|
| `<TO>`   | The account to transfer ckBTC to. |

## Flags

| Flag           | Description                      |
|----------------|----------------------------------|
| `-h`, `--help` | Displays usage information.      |
| `--testnet`    | Uses ckTESTBTC instead of ckBTC. |

## Options

| Option                                | Description                                   |
|---------------------------------------|-----------------------------------------------|
| `--amount <AMOUNT>`                   | The amount, in decimal ckBTC, to transfer.    |
| `--fee <FEE>`                         | The expected fee for this transaction.        |
| `--from-subaccount <FROM_SUBACCOUNT>` | The subaccount to transfer ckBTC from.        |
| `--memo <MEMO>`                       | An integer memo for this transaction.         |
| `--satoshis <SATOSHIS>`               | The amount, in integer satoshis, to transfer. |
| `--to-subaccount <TO_SUBACCOUNT>`     | The subaccount to transfer to.                |
