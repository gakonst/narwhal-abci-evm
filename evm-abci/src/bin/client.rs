use abci::types::Header;
use ethers::prelude::*;
use eyre::Result;

use tendermint_abci::ClientBuilder;
use tendermint_proto::abci::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = ClientBuilder::default().connect("127.0.0.1:26658").unwrap();
    let res = client.info(RequestInfo::default()).unwrap();
    let height = res.last_block_height + 1;
    println!("RequestInfo: {:?}", res);

    let mut client = ClientBuilder::default().connect("127.0.0.1:26658").unwrap();
    match client.init_chain(RequestInitChain::default()) {
        Ok(res) => println!("{:?}", res),
        Err(err) => println!("Skipping, already initialized {:?}", err),
    };

    let mut client = ClientBuilder::default().connect("127.0.0.1:26658").unwrap();
    let res2 = client
        .query(RequestQuery {
            data: "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f".into(),
            path: "".to_string(),
            height,
            prove: false,
        })
        .unwrap();
    let bob_balance = U256::from_little_endian(&res2.value);
    println!("Balance before {:?}", bob_balance);

    let mut client = ClientBuilder::default().connect("127.0.0.1:26658").unwrap();
    let begin_block = RequestBeginBlock {
        header: Some(Header {
            height,
            app_hash: res.last_block_app_hash,
            ..Default::default()
        }),
        ..Default::default()
    };
    if let Err(err) = client.begin_block(begin_block) {
        println!("Not in beginblock state {:?}", err);
    } else {
        println!("BeginBlock");
    }

    client
        .deliver_tx(RequestDeliverTx {
            tx: serde_json::to_vec(
                &TransactionRequest::new()
                    .from(
                        "0x8fd379246834eac74b8419ffda202cf8051f7a03"
                            .parse::<Address>()
                            .unwrap(),
                    )
                    .to("0x88f9b82462f6c4bf4a0fb15e5c3971559a316e7f"
                        .parse::<Address>()
                        .unwrap())
                    .value(500u64)
                    .gas(40000u64)
                    .gas_price(875000000u64),
            )
            .unwrap(),
        })
        .unwrap();

    client.end_block(RequestEndBlock { height })?;

    let mut client = ClientBuilder::default().connect("127.0.0.1:26658").unwrap();
    client.commit()?;

    let mut client = ClientBuilder::default().connect("127.0.0.1:26658").unwrap();
    let res = client
        .query(RequestQuery {
            data: "88f9b82462f6c4bf4a0fb15e5c3971559a316e7f".into(),
            path: "".to_string(),
            height,
            prove: false,
        })
        .unwrap();
    let bob_balance = U256::from_little_endian(&res.value);
    println!("Balance after {:?}", bob_balance);

    println!("END");

    Ok(())
}
