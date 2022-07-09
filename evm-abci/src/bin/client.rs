use ethers::prelude::*;
use evm_abci::types::{Query, QueryResponse};
use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // the ABCI port
    let host = "http://127.0.0.1:3002";

    let val = ethers::utils::parse_units(1, 18).unwrap();
    let alice = "0x2c47Faeff6ED706E9C1D95f51F1800938Cf7c632".parse::<Address>()?;
    let bob = Address::random();

    let _tx = TransactionRequest::new()
        .from(alice)
        .to(bob)
        .value(val)
        .gas(21000);

    let client = reqwest::Client::new();

    let query = Query::Balance(alice);
    let query = serde_json::to_vec(&query).unwrap();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", hex::encode(query)), ("path", "".to_string())])
        .send()
        .await?;
    let val = res.bytes().await?;
    dbg!(&val);
    let val: QueryResponse = serde_json::from_slice(&val)?;
    let balance = val.as_balance();

    println!("Alice balance before: {}", balance);

    // Query balance before

    // Execute transaction

    // Query balance after

    Ok(())
}
