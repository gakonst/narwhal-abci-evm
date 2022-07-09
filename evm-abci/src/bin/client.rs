use ethers::prelude::*;
use evm_abci::types::{Query, QueryResponse};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // the ABCI port
    let host = "http://127.0.0.1:3002";

    let value = ethers::utils::parse_units(1, 18).unwrap();
    let alice = "0x2c47Faeff6ED706E9C1D95f51F1800938Cf7c632".parse::<Address>()?;
    let bob = "0xAAAAAAAAAAAAAABBBBBBBBBBBBBBBB938Cf7c632".parse::<Address>()?;

    let client = reqwest::Client::new();

    // Query balance before
    let query = Query::Balance(alice);
    let query = serde_json::to_string(&query).unwrap();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", query), ("path", "".to_string())])
        .send()
        .await?;
    let val = res.bytes().await?;
    let val: QueryResponse = serde_json::from_slice(&val)?;
    let balance = val.as_balance();
    println!("Alice balance before: {}", balance);

    let query = Query::Balance(bob);
    let query = serde_json::to_string(&query).unwrap();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", query), ("path", "".to_string())])
        .send()
        .await?;
    let val = res.bytes().await?;
    let val: QueryResponse = serde_json::from_slice(&val)?;
    let balance = val.as_balance();
    println!("Bob balance before: {}", balance);

    // Execute transaction
    let tx = TransactionRequest::new()
        .from(alice)
        .to(bob)
        .value(value)
        .gas(21000);

    let tx = serde_json::to_string(&tx)?;
    let res = client
        .get(format!("{}/broadcast_tx", host))
        .query(&[("tx", tx)])
        .send()
        .await?;

    // Takes ~5 seconds to actually apply the state transition?
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Query balance after
    let query = Query::Balance(alice);
    let query = serde_json::to_string(&query).unwrap();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", query), ("path", "".to_string())])
        .send()
        .await?;
    let val = res.bytes().await?;
    let val: QueryResponse = serde_json::from_slice(&val)?;
    let balance = val.as_balance();
    println!("Alice balance after: {}", balance);

    let query = Query::Balance(bob);
    let query = serde_json::to_string(&query).unwrap();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", query), ("path", "".to_string())])
        .send()
        .await?;
    let val = res.bytes().await?;
    let val: QueryResponse = serde_json::from_slice(&val)?;
    let balance = val.as_balance();
    println!("Bob balance after: {}", balance);

    Ok(())
}
