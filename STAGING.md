# STAGING IS LIVE 🎉

You can now interact with a persistent node, deployed on the Starknet Sepolia testing environment.
Sadly, for now, the best way to do so is through a command-line tool.
Here is a little guide on how to do that.

## Installation

### Clone the repository

```shell
git clone https://github.com/nutty-raccoon/paynet
```

### Build the binaries

You won't need to build the node (it is already deployed), just the CLI wallet and a second helper binary that makes depositing money easier.

```shell
cargo build -p starknet-on-chain-setup --bin starknet-on-chain-setup
```
and
```shell
cargo build -p cli-wallet --bin cli-wallet --no-default-features --features=tls
```

## Using Paynet

### Wallet DB

By default, the CLI will create your wallet (which is just a SQLite file) under your system data directory.
E.g., "/Users/tdelabro/Library/Application Support/cli-wallet.sqlite3"

You can modify this behavior by prefixing any command with the `--db-path` argument, as follows:
```shell
./target/debug/cli-wallet --db-path /tmp/my_db.sqlite3 balance
```

For the purpose of this tutorial, I suggest you keep the default configuration.

### Add the node

First, you want to register the node URL as one of the nodes supported by your wallet.

```shell
./target/debug/cli-wallet node add --node-url "https://tdelabro.com"
```

"tdelabro.com" is where I deployed the node I'm running.
"https" is used to communicate over TLS.

It should display the new ID of this node, a number, most likely `1` as this is your first node.

You can confirm this by running the following command:

```shell
./target/debug/cli-wallet node ls
```

It should display the following message:
```
Available nodes
1 https://tdelabro.com/
```
listing your known node ID and URL.

### Consulting your balances

You can see how much money is stored in your wallet by running:

```shell
./target/debug/cli-wallet balance
```

For now, it's empty. Let's fix that.

### Depositing money

Time to deposit some Sepolia STRK on the node.
This is the most complex operation of the flow, so let's go through it attentively.

```shell
./target/debug/cli-wallet mint new --amount 100 --asset strk --node-id 1
```
which should print something like this:
```shell
Requesting https://tdelabro.com/ to mint 100 strk
MintQuote created with id: b66f12b8-d137-4b49-a541-0cce340d6a6a
Proceed to payment:
[
  {
    "to":"0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d",
    "selector":"0x219209e083275171774dab1df80982e9df2096516f06319c5c6d71ae0a8480c",
    "calldata":["0x44aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46","0x56bc75e2d63100000","0x0"]
    },
    {
    "to":"0x44aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46",
    "selector":"0xd5c0f26335ab142eb700850eded4619418b0f6e98c5b92a6347b68d2f2a0c",
    "calldata":["0x503919cb4a0191552f007c190fc3f78e63b912f1500c11c8aeda04b5b18a6b6","0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d","0x56bc75e2d63100000","0x0","0x2a4c56a99f93d0b19f9a3b09640cb9fd1f4c426474a85dedfec573849ab6235"]
  }
]
```
and then keep your terminal busy. It's waiting for you to deposit money on the node, on-chain.
The JSON you see contains the two calls you will have to sign to correctly deposit money.
We are going to do that in a single transaction very soon, but first, let's take a look at them.

The first asks your wallet to allow the contract at `0x44aa20c51f815974487cbe06ae547a16690d4ca7f8c703aa8bbffe6d7393d46` to spend 100 (`"0x56bc75e2d63100000","0x0"` as u256) STRK in your name. 
This is effectively a call to the `approve` entrypoint listed here: https://sepolia.voyager.online/contract/0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d#writeContract.

The second one is a call to the contract you just gave permission to spend your money. It calls its `pay_invoice` selector, which takes a few arguments: `invoice_id` (which is a hash of the quote ID we got before), `asset` (the address of the STRK token contract), `amount` (here `100`), and `payee` (the recipient of the transfer). 
This whole contract, which you can consult [here](./contracts/invoice/src/lib.cairo), is just a wrapper over an ERC20 transfer with a slightly more complex event, including the `invoice_id`, which is what the node listens to in order to register your invoice as paid.

Now that we know what we are signing, let's sign it.

1. Copy the whole JSON string
2. Open a new terminal
3. `export CALLS='<ctrl+V>'`
4. Run our helper binary:
```shell
 ./target/debug/starknet-on-chain-setup \
  --chain-id="SN_SEPOLIA" \
  --url="https://starknet-sepolia.public.blastapi.io" \
  --private-key=<your private key> \
  --account-address=<your account address> \
  pay-invoice --invoice-json-string $CALLS
```

For this purpose, I suggest you create a local account using Starkli, but you could also use your main Argent or Braavos wallet.
Be careful though, this program, if malicious, could drain your account. Fortunately, you compiled it yourself, based on [source code](./crates/bin/starknet-on-chain-setup/src/main.rs) you can read.
Alternatively, you can pay the invoice using any other way you like; just be cautious while copying the payload.

If you happen to lose connection to the node while the CLI is waiting for the deposit, don't worry, you can run the following command:
```shell
./target/debug/cli-wallet mint sync
```
It will query the nodes about all your ongoing mint operations, and if one was recently paid, download the new tokens locally.

### Send

Now that you have some tokens, which you can see by running:
```shell
./target/debug/cli-wallet balance
```
you can send some to someone.

To do so, run:
```shell
./target/debug/cli-wallet send --node-id 1 --amount 10 --asset strk -o ./10strk.wad
```

It will store the money inside the `10strk.wad` file.
The content will be something like this:
`paynetBo2FudWh0dHBzOi8vdGRlbGFicm8uY29tL2F1aW1pbGxpc3Rya2FwgaJhaUgAe32dgxHDg2FwiKNhYRhAYXN4QDMwODBmYWM1NzNjOGQwM2E4OWEzNDk4ZGI5YTdmNTNlNThjMWZkMGUzNTMxNjRiZTM0MzVhNGU4MTFiZTdlOWNhY1ghArMvhY0BwKhC9kirBTeHQ_-B7jtVxFK6lk25g_APehZTo2FhGQIAYXN4QGJiZWUyN2RjM2NiMTU1YmVkYmE4YTVmMWQ1Yzk5MzMxMjZjMjNiZjk1Yjc3Y2M4MTJmMDRiY2I5OTI1Njk5MDNhY1ghAlPMA699HM2ziL5q5Uv6HNJrK4O1cmZI26PxZ0cJ8mnCo2FhGCBhc3hAMjUzM2FiZGZmMzVjNTY1OGNlZWVlNjYwZWM3MDFjMzMyMzkxNDUyNGI5YmNiOGQzYmNjMDNiYzk4MmU5ZGFhZGFjWCECdUz7IJ_pwLV30GYLAEzC5cwS2xhELZI1mEumcmDClyCjYWEYIGFzeEA1NjE4NDIyYjU4YTYzODA4ZjI1NjVkZWI1YTgxYjY3NGQ1NWY3OTU3YmMyYTJkM2RmNDI2MDBlYzRkODVhYjYxYWNYIQI6aI4JW6OPhtW8vvFaNEXJC_qbU6mVktfdV8o6pMEAMqNhYRiAYXN4QDdkMWI1OTk5YWM3NmE1ZjE3ZGI4MDA5YTI2NmEzYzc3OTkwNmFmZWUyMmMwMDhiOTRhMGY0NzFkOTkzOWRkNTlhY1ghAgYvjD7_tskcy-VA4Ku5C1sWvEqwsM8RJa78l1cqqDBSo2FhGQQAYXN4QDg2NGI3MDM5YjBkMGNiNjQ3YzIyYWNjM2NiZjQ4NzNjOGYyMWUxNDY3YzA5MDAzYjE2NmI1NjFmYjhkZTVhMWJhY1ghA9H_ZJm7QSIEsOCCozDQv50a4dxM3AkvF2eTMO-ypSwPo2FhEGFzeEA5OThmNDc5YzIwYWMzYjMwZDQ2NzA5ZjFiZTBhZGExZTE0MGZmZjBmMjE5N2U3NzZlOTRkZWY0NzM3NDM3ODc1YWNYIQMUeTlhne6DeD0CPOSDVBZSu80qedRMOa3tuOBY46EofaNhYRkgAGFzeEBmZWM5YzdmNmE0YjNhMzk1YzI1N2Q1YWM1ZDM3ZjEyMGFjMjQxYzIzNTA3NjY3NmI1OWIxZWQ3ZWQyNWU3NWM2YWNYIQJQq_DkoByxvwsN2YoGwPluHSD-aKlCLk_HpXoNx6YHmw==`

You can decode it to a more readable form by running:
```shell
./target/debug/cli-wallet decode-wad -f ./10strk.wad
```
It will display a JSON containing a `node_url` as well as a list of proofs, each one for a specific amount, with the total adding up to the amount you decided to send.

If you run `balance` again, you will see that it has decreased because the money is no longer stored in your database; it is now stored inside this new file.
You can send this file to a friend, a shop, or consume it yourself. And you can do so by email, Signal, Telegram, Airdrop, USB drive, or any way you feel is appropriate.
As long as you can chat, you can pay.

### Receive

Let's say someone sent such a file to you; how do you consume it?
Like this:
```shell
 ./target/debug/cli-wallet receive -f ./10strk.wad
```

Now, check your balance; it has increased.
If you try to receive the file again, you will get a "proof already used" error. No double spending is allowed here!
You can remove the `10strk.wad` file if you wish; it is not of any use anymore.

### Withdrawing money

Now let's take your money out of your local wallet and back on-chain.
```shell
./target/debug/cli-wallet melt --node-id 1 --amount 10 --asset strk --to <your on-chain contract address>
```
It will print the transaction hash of the on-chain transfer to your wallet, and soon enough your money will be on your account.
Check your balance; it has been updated too.

## Final words

Thanks for trying out this product. Your feedback is very precious to us, so don't hesitate to write me a message at @tdelabro on Telegram.
