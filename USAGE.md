# Development

This document describes the process for running this application on your local computer.

## Getting started

Install rust: https://www.rust-lang.org/tools/install

Install docker: https://docs.docker.com/desktop/

## Run the node

Using docker you will be able to run all the required services locally.

```shell
$ docker-compose -p paynet -f ./docker-compose.yml up -d
```

## Interact with the node

### Build the wallet

You can use the CLI wallet to interact with the node.

```shell
$ cargo build -p cli-wallet
```

then

```shell
$ ./target/debug/cli-wallet -h
```

### End-to-end user flow

The end-to-end flow would look like this.

#### Register node

```shell
$ cli-wallet node add -n "http://[::1]:20001"
```

`[::1]` is localhost, and `20001` is the value used it the `docker-compose.yml` file.

You can then list the available nodes:

```shell
$ cli-wallet node ls
```

#### Mint

```shell
$ cli-wallet mint -n 1 -a 50 -u strk
```

By default and for local development purposes, the tokens are not backed by any on-chain asset,
so they will be minted immediately, without having to wait for a deposit.

#### Send

```shell
$ cli-wallet cli-wallet send -n 1 -a 42 -u strk
```

This will print a `wad` of tokens, as a json:

```json
{
  "node_url": "http://[::1]:20001",
  "proofs": [
    {
      "amount": 32,
      "id": "0067e74b82ff0fe9",
      "secret": "d1106d1f171246b13d0ee853d8e46de29faeb89080ed1ab4c549f7f3b09acec6",
      "C": "0287cfb3f21fba337bec0211e24ae52bfa2a9b70b8cb303072b3c33c8a45454050"
    },
    {
      "amount": 2,
      "id": "0067e74b82ff0fe9",
      "secret": "2c5481dcd74ae7dfe4bc29b9bc20cfc8acc581033ef35414a31d6525a219e2dd",
      "C": "028e07d021e984590a568852b0f3bc40d77450cae5b45ad69eabcf5c51e171922c"
    },
    {
      "amount": 8,
      "id": "0067e74b82ff0fe9",
      "secret": "cecd1e6daebe1621c98e9b912a2ffa99a49a985b6c64364c9613bca50b7c4454",
      "C": "03cc27e99284824c0cbce568989a86e4d6ef03614c161e1728848a2bdb9d000c0d"
    }
  ]
}
```

Copy it and save it somewhere.

#### Receive

```shell
$ cli-wallet cli-wallet receive -w '<the wad json content>'
```

This could be run on the same or a different wallet.

#### Melt

```shell
$ cli-wallet cli-wallet melt -n 1 -a 21 -u strk
```

If we were running the on-chain logic, you would have to specify the address on which you want to withdraw,
and wait for the transaction to be processed.
