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
  -g, --path-genesis-head <PATH_GENESIS_HEAD>        Path to a file with a genesis head
  -v, --path-validation-code <PATH_VALIDATION_CODE>  Path to the wasm file
  -h, --help                                         Print help
```

Run locally:
```shell
cargo run para_id account_address -g genesis_head -v validation_code
```
The metadata used for subxt has been queried with:
```shell
subxt metadata --url https://rpc.polkadot.io:443 -f bytes > metadata/polkadot_metadata.scale
subxt metadata --url https://kusama-rpc.polkadot.io:443 -f bytes >  metadata/kusama_metadata.scale
subxt metadata --url https://rococo-rpc.polkadot.io:443 -f bytes >  metadata/rococo_metadata.scale
subxt metadata -f bytes >  metadata/local_metadata.scale  
```

### Secret Keys
Create a `.env` file and add the variable `SEED` with your seed key like in the `.env.example` file
This allows you to do queries into Rococo (as a sudo)