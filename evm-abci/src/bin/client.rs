use ethers::prelude::*;
use evm_abci::types::{Query, QueryResponse};
use eyre::Result;

async fn query_balance(host: &str, address: Address) -> U256 {
    let query = Query::Balance(address);
    let query = serde_json::to_string(&query).unwrap();

    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", query), ("path", "".to_string())])
        .send()
        .await
        .unwrap();

    let val = res.bytes().await.unwrap();
    let val: QueryResponse = serde_json::from_slice(&val).unwrap();
    val.as_balance()
}

async fn send_transaction(host: &str, from: Address, to: Address, value: U256) {
    let tx = TransactionRequest::new()
        .from(from)
        .to(to)
        .value(value)
        .gas(21000);

    let tx = serde_json::to_string(&tx).unwrap();

    let client = reqwest::Client::new();
    client
        .get(format!("{}/broadcast_tx", host))
        .query(&[("tx", tx)])
        .send()
        .await
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    // the ABCI port on the various narwhal primaries
    let host_1 = "http://127.0.0.1:3002";
    let host_2 = "http://127.0.0.1:3009";
    let host_3 = "http://127.0.0.1:3016";

    let value = ethers::utils::parse_units(1, 18).unwrap();
    let alice = "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".parse::<Address>()?;
    let bob = "0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".parse::<Address>()?;
    let charlie = "0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".parse::<Address>()?;

    // Query initial balances from host_1
    println!("Querying initial balances from {}:", host_1);

    let alice_initial_balance = query_balance(host_1, alice).await;
    println!("Alice balance before: {}", alice_initial_balance);

    let bob_initial_balance = query_balance(host_1, bob).await;
    println!("Bob balance before: {}", bob_initial_balance);

    let charlie_initial_balance = query_balance(host_1, charlie).await;
    println!("Charlie balance before: {}", charlie_initial_balance);

    println!("---");

    // Send conflicting transactions
    println!("Alice sends conflicting transactions:");
    println!("Alice sends TX to {} where she sends 1 ETH to Bob...", host_2);
    send_transaction(host_2, alice, bob, value).await;
    println!("Alice sends TX to {} where she sends 1 ETH to Charlie...", host_3);
    send_transaction(host_3, alice, charlie, value).await;

    println!("---");

    // Takes ~5 seconds to actually apply the state transition?
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Query final balances from host_2
    println!("Quering final balances from {}:", host_2);

    let alice_final_balance_host_2 = query_balance(host_2, alice).await;
    println!("Alice balance after: {}", alice_final_balance_host_2);

    let bob_final_balance_host_2 = query_balance(host_2, bob).await;
    println!("Bob balance after: {}", bob_final_balance_host_2);

    let charlie_final_balance_host_2 = query_balance(host_2, charlie).await;
    println!("Charlie balance after: {}", charlie_final_balance_host_2);

    println!("---");

    // Query final balances from host_3
    println!("Quering final balances from {}:", host_3);

    let alice_final_balance_host_3 = query_balance(host_3, alice).await;
    println!("Alice balance after: {}", alice_final_balance_host_3);

    let bob_final_balance_host_3 = query_balance(host_3, bob).await;
    println!("Bob balance after: {}", bob_final_balance_host_3);

    let charlie_final_balance_host_3 = query_balance(host_3, charlie).await;
    println!("Charlie balance after: {}", charlie_final_balance_host_3);

    Ok(())
}
