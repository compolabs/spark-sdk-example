# Spark SDK Examples

This repository contains examples demonstrating the usage of CLOB Spark on the Fuel network. There are two tests included: one for the local node and another for the testnet beta-3.

## ⚠️ Warning
Please note that we are transitioning to a new beta-4 test version. Therefore, it is essential to use specific version of the fuel toolchain. 

You can install this toolchain using `forc toolchain`:
```
fuelup toolchain new my-custom-toolchain  
fuelup component add forc@0.39.0   
fuelup component add fuel-core@0.17.13 
```

Ensure that you have the following versions installed, you can check the versions using `fuelup show`:
```
>>> fuelup show
Default host: aarch64-apple-darwin
fuelup home: /Users/alexey/.fuelup

installed toolchains
--------------------
beta-3-aarch64-apple-darwin
latest-aarch64-apple-darwin
hotfix
my-custom-toolchain (default)

active toolchain
-----------------
my-custom-toolchain (default)
  forc : 0.39.0
    - forc-client
      - forc-deploy : 0.39.0
      - forc-run : 0.39.0
    - forc-doc : 0.39.0
    - forc-explore - not found
    - forc-fmt : 0.39.0
    - forc-index - not found
    - forc-lsp : 0.39.0
    - forc-tx : 0.39.0
    - forc-wallet - not found
  fuel-core : 0.17.13
  fuel-indexer - not found

fuels versions
---------------
forc : 0.39
```

## Tests
- localnode_example.rs
- testnet_example.rs

## Additional Information
You can find more documentation on our SDKs here:
- Spark Rust SDK
- SRC-20 Rust SDK