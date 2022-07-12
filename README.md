# Narwhal & Bullshark x ABCI x EVM

Components
* Reliable stream of finalized transaction hashes from Bullshark
* Reconstruction of the Ledger by querying Narwhal workers' stores
* Delivery of the reconstructed ledger over ABCI
* Implementation of a Rust ABCI app using REVM

![](./assets/architecture.png)

## Demo

1. `cargo run --bin evm-app`
2. 2nd terminal: `cd demo && poetry install`
3. 3rd terminal: `cargo run --bin client`

This will spin up a network instance with the EVM app in Demo mode (giving Alice 1.5 ETH) and transfer 1 ETH from Alice to Bob. This uses the underlying Foundry EVM.

## TODOs

1. Why does the state transition take a few seconds to get applied?
2. Can we make this work with Anvil instead of rebuilding a full evm execution env?
