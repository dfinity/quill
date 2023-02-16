# quill ckbtc retrieve-btc

Signs messages to retrieve BTC in exchange for ckBTC.

This command generates two messages by default; a transfer of ckBTC to the minting canister, and a request for BTC. However, if you have already made this transfer (the address can be viewed with `quill ckbtc get-withdrawal-account`), you can use the `--already-transferred` flag to skip the first message.

Bitcoin transactions take a while, so the response to the second message will not be a success state, but rather a block index. Use the `quill ckbtc retrieve-btc-status` command to check the status of this transfer.

## Basic usage

The basic syntax for running `quill ckbtc retrieve-btc` commands is:

```bash
quill ckbtc retrieve-btc [option] <TO>
```

## Arguments

| Argument | Description                                                                             |
|----------|-----------------------------------------------------------------------------------------|
| `<TO>`   | The Bitcoin address to send the BTC to. Note that Quill does not validate this address. |

## Flags

| Flag                    | Description                                                            |
|-------------------------|------------------------------------------------------------------------|
| `--already-transferred` | Skips signing the transfer of ckBTC, signing only the request for BTC. |
| `-h`, `--help`          | Displays usage information.                                            |
| `--testnet`             | Uses ckTESTBTC instead of ckBTC.                                       |

## Options

| Option                                | Description                                    |
|---------------------------------------|------------------------------------------------|
| `--amount <AMOUNT>`                   | The quantity, in decimal BTC, to convert.      |
| `--fee <FEE>`                         | The expected fee for the ckBTC transfer.       |
| `--from-subaccount <FROM_SUBACCOUNT>` | The subaccount to transfer the ckBTC from.     |
| `--memo <MEMO>`                       | An integer memo for the ckBTC transfer.        |
| `--satoshis <SATOSHIS>`               | The quantity, in integer satoshis, to convert. |
