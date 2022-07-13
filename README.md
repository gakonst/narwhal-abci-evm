# Narwhal & Bullshark x ABCI x EVM

Components:
* Reliable stream of hashes of batches of transactions from Bullshark
* Reconstruction of the ledger by querying Narwhal workers' stores for the confirmed batches of transactions
* Delivery of the reconstructed ledger over ABCI to the application logic
* Implementation of a Rust ABCI app using REVM

![](./assets/architecture.png)

## Demo

Setup/dependencies (from the main folder of the repository):
* [Rust](https://www.rust-lang.org/)
* [Python Poetry](https://python-poetry.org/)
* [tmux](https://github.com/tmux/tmux)
* `cd demo && poetry install`

Run demo (from the main folder of the repository):
2. 1st terminal: `cd demo && cargo build && poetry run fab local`
3. 2nd terminal (after the testbed has started in 1st terminal): `cargo run --bin client`

The second command will produce output like this:
<script id="asciicast-DP9RN2FzEtIyndGQdFxdkHXen" src="https://asciinema.org/a/DP9RN2FzEtIyndGQdFxdkHXen.js" async></script>

The demo consensus network is run by four nodes (each running on localhost), whose RPC endpoints are reachable on TCP ports 3002, 3009, 3016, and 3023, respectively. There are three accounts, Alice (initially 1.5 ETH), Bob (initially 0 ETH), and Charlie (initially 0 ETH). Alice performs a double spend, sending 1 ETH each to Bob and Charlie in two different transactions that get input to the nodes at ports 3009 and 3016, respectively. Note that only one transaction can make it. Eventually, nodes reach consensus on which transaction gets executed in Foundry's EVM, and the application state is updated in lockstep across all nodes. The update is reflected in subsequent balance queries.

## TODOs

1. Why does the state transition take a few seconds to get applied?
2. Can we make this work with Anvil instead of rebuilding a full EVM execution environment?
