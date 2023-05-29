# CLI for Onboarding Parachains
CLI tool to onboard parachains.

Build:
```shell
cargo build
```

Install the tool locally:
```shell
cargo install --path .
```

How to use:
```
Usage: para-onboarding <PARA_ID> <ACCOUNT_ADDRESS>

Arguments:
  <PARA_ID>          Parachain ID
  <ACCOUNT_ADDRESS>  Manager Address

Options:
  -h, --help  Print help
```

Run locally:
```shell
cargo run para_id account_address
```
The metadata used for subxt has been queried with:
```shell
subxt metadata --url https://rpc.polkadot.io:443 -f bytes > polkadot-metadata.scale
subxt metadata --url https://kusama-rpc.polkadot.io:443 -f bytes > kusama-metadata.scale
subxt metadata --url https://rococo-rpc.polkadot.io:443 -f bytes > rococo-metadata.scale
```