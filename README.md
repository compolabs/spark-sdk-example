# Spark SDK Examples

This repository contains examples demonstrating the usage of CLOB Spark on the Fuel network. There are two tests included: one for the local node and another for the testnet beta-3.

## ⚠️ Warning
Please note that we are transitioning to a new beta-4 test version. Therefore, it is essential to use specific versions of the toolchain and Rust SDK. Ensure that you have the following versions installed:

### Rust SDK:
```toml
fuels = { git = "https://github.com/FuelLabs/fuels-rs.git", branch = "hotfix/v0.41.0-predicate-configurables" }
```

### Toolchain:
```
forc: 0.39.0
  - forc-client
    - forc-deploy: 0.39.0
    - forc-run: 0.39.0
  - forc-doc: 0.39.0
  - forc-fmt: 0.39.0
  - forc-lsp: 0.39.0
  - forc-tx: 0.39.0
fuel-core: 0.17.13
```

## Tests
- localnode_example.rs
- testnet_example.rs

## Additional Information
You can find more documentation on our SDKs here:
- Spark Rust SDK
- SRC-20 Rust SDK