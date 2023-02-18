# quill public-ids

Prints the principal id and the account id.

## Basic usage

The basic syntax for running `quill public-ids` commands is:

``` bash
quill public-ids [option]
```

## Flags

| Flag           | Description                 |
|----------------|-----------------------------|
| `-h`, `--help` | Displays usage information. |

## Options

| Option                          | Description                                |
|---------------------------------|--------------------------------------------|
| `--principal-id <PRINCIPAL_ID>` | Principal for which to get the account_id. |

## Examples

The `quill public-ids` command is used to display your principal and account ID, or the ID of a given principal.

For example, to view the default account ID for the anonymous principal:

```sh
quill public-ids --principal-id 2vxsx-fae
```

This will produce the output:

```
Principal id: 2vxsx-fae
Account id: 1c7a48ba6a562aa9eaa2481a9049cdf0433b9738c992d698c31d8abf89cadc79
```
