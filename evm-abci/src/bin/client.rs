use ethers::prelude::*;
use evm_abci::types::{Query, QueryResponse};
use eyre::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use yansi::{Paint};

static ALICE: Lazy<Address> = Lazy::new(||{
    "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".parse::<Address>().unwrap()
});
static BOB: Lazy<Address> = Lazy::new(||{
    "0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB".parse::<Address>().unwrap()
});
static CHARLIE: Lazy<Address> = Lazy::new(||{
    "0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC".parse::<Address>().unwrap()
});

static ADDRESS_TO_NAME: Lazy<HashMap<Address, &'static str>> = Lazy::new(||{
    let mut address_to_name = HashMap::new();
    address_to_name.insert(*ALICE, "Alice");
    address_to_name.insert(*BOB, "Bob");
    address_to_name.insert(*CHARLIE, "Charlie");

    address_to_name
});


fn get_readable_eth_value(value: U256) -> Result<f64> {
    let value_string = ethers::utils::format_units(value, "ether")?;
    Ok(value_string.parse::<f64>()?)
}

async fn query_balance(host: &str, address: Address) -> Result<()> {
    let query = Query::Balance(address);
    let query = serde_json::to_string(&query)?;

    let client = reqwest::Client::new();
    let res = client
        .get(format!("{}/abci_query", host))
        .query(&[("data", query), ("path", "".to_string())])
        .send()
        .await?;

    let val = res.bytes().await?;
    let val: QueryResponse = serde_json::from_slice(&val)?;
    let val = val.as_balance();
    let readable_value = get_readable_eth_value(val)?;
    let name = ADDRESS_TO_NAME.get(&address).unwrap();
    println!(
        "{}'s balance: {}",
        Paint::new(name).bold(),
        Paint::green(format!("{} ETH", readable_value)).bold()
    );
    Ok(())
}

async fn query_all_balances(host: &str) -> Result<()> {
    println!(
        "Querying balances from {}:",
        Paint::new(format!("{}", host)).bold()
    );

    query_balance(host, *ALICE).await?;
    query_balance(host, *BOB).await?;
    query_balance(host, *CHARLIE).await?;

    Ok(())
}

async fn send_transaction(host: &str, from: Address, to: Address, value: U256) -> Result<()> {
    let from_name = ADDRESS_TO_NAME.get(&from).unwrap();
    let to_name = ADDRESS_TO_NAME.get(&to).unwrap();
    let readable_value = get_readable_eth_value(value)?;
    println!(
        "{} sends TX to {} transferring {} to {}...",
        Paint::new(from_name).bold(),
        Paint::red(host).bold(),
        Paint::new(format!("{} ETH", readable_value)).bold(),
        Paint::red(to_name).bold()
    );

    let tx = TransactionRequest::new()
        .from(from)
        .to(to)
        .value(value)
        .gas(21000);

    let tx = serde_json::to_string(&tx)?;

    let client = reqwest::Client::new();
    client
        .get(format!("{}/broadcast_tx", host))
        .query(&[("tx", tx)])
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // the ABCI port on the various narwhal primaries
    let host_1 = "http://127.0.0.1:3002";
    let host_2 = "http://127.0.0.1:3009";
    let host_3 = "http://127.0.0.1:3016";

    let value = ethers::utils::parse_units(1, 18)?;

    // Query initial balances from host_1
    query_all_balances(host_1).await?;

    println!("---");

    // Send conflicting transactions
    println!(
        "{} sends {} transactions:",
        Paint::new("Alice").bold(),
        Paint::red(format!("conflicting")).bold()
    );
    send_transaction(host_2, *ALICE, *BOB, value).await?;
    send_transaction(host_3, *ALICE, *CHARLIE, value).await?;

    println!("---");

    println!("Waiting for consensus...");
    // Takes ~5 seconds to actually apply the state transition?
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    println!("---");

    // Query final balances from host_2
    query_all_balances(host_2).await?;

    println!("---");

    // Query final balances from host_3
    query_all_balances(host_3).await?;

    Ok(())
}
