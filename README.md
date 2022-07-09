# Narwhal & Bullshark x ABCI x EVM

Components
* Reliable stream of finalized transaction hashes from Bullshark
* Reconstruction of the Ledger by querying Narwhal workers' stores
* Delivery of the reconstructed ledger over ABCI
* Implementation of a Rust ABCI app using REVM

![](./assets/architecture.png)
