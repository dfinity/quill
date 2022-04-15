# sns-quill

Cold wallet toolkit for interacting with a Service Nervous System's Ledger & Governance canisters.

## Disclaimer

YOU EXPRESSLY ACKNOWLEDGE AND AGREE THAT USE OF THIS SOFTWARE IS AT YOUR SOLE RISK.
AUTHORS OF THIS SOFTWARE SHALL NOT BE LIABLE FOR DAMAGES OF ANY TYPE, WHETHER DIRECT OR INDIRECT.

## Usage

---

### Generating Identities

Most `sns-quill` commands will require an Internet Computer compatible identity. As of version 0.1.0 `sns-quill` only
supports an identity via PEM format, however, `sns-quill` provides the ability to generate these identities in a secure
manner.

To generate a seed phrase and a PEM file, run the following command.

```shell
$ sns-quill generate --pem-file identity.pem --seed-file seed.txt
```

The generated seed.txt is the mnemonic BIP39 representation of the private key in identity.pem. `sns-quill` will take
the identity.pem file as input when signing messages. The above command also outputs the PrincipalId and AccountId of
the private key. To get those public ids from a given PEM file, run the following command:

```shell
$ sns-quill --pem-file identity.pem public-ids

Principal id: zukuq-h6vpk-jp4g2-j37gz-vnm7b-34yi7-zehhn-zqwks-vzzms-dvsw4-5qe
Account id: 40c7abdde72cb8b7838bba9290e78c8e6819507763447f38afaa2a07a82dafe0
```

---

### Signing and Sending Messages

`sns-quill` is a toolkit for interacting with the Service Nervous System's (SNS) canisters using self-custody keys. These
keys can be held in an air-gapped computer (a computer that has never connected to the internet) known as a cold wallet.
To support cold wallets, `sns-quill` takes a two-phase approach to sending query/update calls to the Internet Computer.
In the first phase, `sns-quill` is used with the various subcommands to generate and sign messages based on user input,
without needing access to the internet. In the second phase, the signed message(s) is sent to the Internet Computer.
Since this requires connection to boundary nodes, cold-wallet users will transport the signed message(s) from the
air-gapped computer (i.e. with a USB stick) to a computer connected with the internet


To route messages to the correct canister, the various SNS CanisterIds will need to be known ahead of time, and
inputted to each subcommand. `sns-quill` has a required flag (`--canister-ids-file`) that reads a JSON file and
parses the ids for later use. An example JSON file would look like this. **Note** if the CanisterIds are incorrect,
signing will succeed but submitting the messages to the IC will result in a rejected message.

```json
{
  "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
  "ledger_canister_id": "ryjl3-tyaaa-aaaaa-aaaba-cai",
  "root_canister_id": "r7inp-6aaaa-aaaaa-aaabq-cai"
}
```
---

### Ledger Commands

To check the balance of an account on an SNS ledger run the following command. This will query the ledger and must
be run from a computer connected to the Internet.

```shell
$ sns-quill --canister-ids-file <path-to-file> account-balance <account-id>
```

To send tokens to another account on the SNS ledger run the following command. This will sign a transfer transaction
and print to STDOUT:

```shell
$ sns-quill --pem-file <path-to-file> --canister-ids-file <path-to-file> transfer <account-id> --amount <amount>
```

To display the signed message in human-readable form:

```shell
$ sns-quill send --dry-run <path-to-file>
```

To send the signed message: 
```shell
$ quill send <path-to-file>
```

---

### Governance Commands

To stake tokens in a neuron, two messages are required: a ledger transfer and a governance claim_or_refresh. 
`sns-quill` provides a single command to accomplish this. 

```shell
$ sns-quill --pem-file <path-to-file> --canister-ids-file <path-to-file> neuron-stake --memo <memo> --amount <amount>
```

A successful submission of this command will result in a ClaimOrRefresh response containing the NeuronId. 
**NOTE**: As of v0.1.0 the NeuronId will be rendered as a list of 8-bit integers. Many of `sns-quill`'s commands require
the NeuronId to be encoded as a hex string. To generate this hex string, copy the list and execute the following command.
Save the output as you will not be able to regenerate the id of your neuron without the original memo.  

```shell
$ export NEURON_ID_ARRAY="[131, 167, 210, 177, 47, 101, 79, 245, 131, 53, 229, 162, 81, 44, 202, 224, 215, 131, 156, 116, 75, 24, 7, 164, 124, 150, 245, 185, 243, 150, 144, 105]"
$ python3 -c "print(bytes($NEURON_ID_ARRAY).hex())"
83a7d2b12f654ff58335e5a2512ccae0d7839c744b1807a47c96f5b9f3969069
```

To extend the dissolve delay of the Neuron, run the following command.

```shell
$ sns-quill --pem-file <path-to-file> --canister-ids-file <path-to-file> configure-dissolve-delay <neuron-id> --additional-dissolve-delay-seconds <seconds>
```

To submit a proposal, run the following command. The proposal must be in submitted as candid.

```shell
$ sns-quill --pem-file <path-to-file> --canister-ids-file <paht-to-file> make-proposal <neuron-id> --proposal <proposal>
```

Below is an example candid Motion proposal.

```
'( 
    record { 
        title="Launch SNS";
        url="https://dfinity.org"; 
        summary="A motion to launch the SNS";
        action=opt variant { 
            Motion=record { 
                motion_text="I hereby raise the motion that the use of the SNS shall commence"; 
            } 
        };  
    } 
)'
```

To vote on a proposal, run the following command.

```shell
$ sns-quill --pem-file <path-to-file> --canister-ids-file <path-to-file> register-vote <neuron-id> --proposal-id <proposal-id> --vote <vote>
```

## Download

Use binaries from the latest [release](https://github.com/DanielThurau/sns-quill/releases).

## Build

To compile `quill` run:

1. `rustup set profile minimal`
2. `rustup toolchain install stable --component rustfmt --component clippy`
3. `rustup override set stable`
4. `make release`

After this, find the binary at `target/release/sns-quill`.

### Building with Nix

If you have Nix installed, you can use it to provide an environment for
running `cargo`. Just replace the above build steps with the following:

To compile `quill` run:

1. `nix-shell`
4. `make release`

After this, find the binary at `target/release/sns-quill`.

## Testnets

If you have access to an Internet Computer testnet (for example, a version the
replica binary and SNS running locally), you can target sns-quill at this test
network by setting the `IC_URL` environment variable to the full URL. For
example:

    alias sns-quill="IC_URL=http://127.0.0.1:8000/ sns-quill"

## Contribution

`sns-quill` is a very critical link in the workflow of the management of valuable assets.
`sns-quill`'s code must stay clean, simple, readable and leave no room for ambiguities, so that it can be reviewed and audited by anyone.
Hence, if you would like to propose a change, please adhere to the following principles:

1. Be concise and only add functional code.
2. Optimize for correctness, then for readability.
3. Avoid adding dependencies at all costs unless it's completely unreasonable to do so.
4. Every new feature (+ a test) is proposed only after it was tested on real wallets.
5. Increment the last digit of the crate version whenever the functionality scope changes. 

## Credit

Originally forked from the [quill](https://github.com/dfinity/quill).
