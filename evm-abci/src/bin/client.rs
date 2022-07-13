use ethers::prelude::*;
use evm_abci::types::{Query, QueryResponse};
use eyre::Result;
use yansi::{Paint};
use crate::User::*;

const ALICE: &str = "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const BOB: &str = "0xBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";
const CHARLIE: &str = "0xCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC";

#[derive(Copy, Clone)]
enum User {
    Alice,
    Bob,
    Charlie
}

fn user_to_address(user: User) -> Result<Address> {
    match user {
        Alice => Ok(ALICE.parse::<Address>()?),
        Bob => Ok(BOB.parse::<Address>()?),
        Charlie => Ok(CHARLIE.parse::<Address>()?),
    }
}

fn user_to_name(user: User) -> &'static str {
    match user {
        Alice => "Alice",
        Bob => "Bob",
        Charlie => "Charlie",
    }
}

fn get_readable_eth_value(value: U256) -> Result<f64> {
    let value_string = ethers::utils::format_units(value, "ether")?;
    Ok(value_string.parse::<f64>()?)
}

async fn query_balance(host: &str, user: User) -> Result<()> {
    let address = user_to_address(user)?;
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
    let name = user_to_name(user);
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

    query_balance(host, Alice).await?;
    query_balance(host, Bob).await?;
    query_balance(host, Charlie).await?;

    Ok(())
}

async fn send_transaction(host: &str, from: User, to: User, value: U256) -> Result<()> {
    let from_name = user_to_name(from);
    let to_name = user_to_name(to);
    let readable_value = get_readable_eth_value(value)?;
    println!(
        "{} sends TX to {} transferring {} to {}...",
        Paint::new(from_name).bold(),
        Paint::red(host).bold(),
        Paint::new(format!("{} ETH", readable_value)).bold(),
        Paint::red(to_name).bold()
    );

    let from_address = user_to_address(from)?;
    let to_address = user_to_address(to)?;
    let tx = TransactionRequest::new()
        .from(from_address)
        .to(to_address)
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

    println!("\n---\n");

    // Send conflicting transactions
    println!(
        "{} sends {} transactions:",
        Paint::new("Alice").bold(),
        Paint::red(format!("conflicting")).bold()
    );
    send_transaction(host_2, Alice, Bob, value).await?;
    send_transaction(host_3, Alice, Charlie, value).await?;

    println!("\n---\n");

    println!("Waiting for consensus...");
    // Takes ~5 seconds to actually apply the state transition?
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    println!("\n---\n");

    // Query final balances from host_2
    query_all_balances(host_2).await?;

    println!("\n---\n");

    // Query final balances from host_3
    query_all_balances(host_3).await?;

    Ok(())
}
