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