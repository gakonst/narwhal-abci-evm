use std::net::SocketAddr;

// Copyright(C) Facebook, Inc. and its affiliates.
use anyhow::{Context, Result};
use clap::{crate_name, crate_version, App, AppSettings, ArgMatches, SubCommand};
use config::Export as _;
use config::Import as _;
use config::WorkerAddresses;
use config::{Committee, KeyPair, Parameters, WorkerId};
use consensus::Consensus;
use env_logger::Env;
use ethers::prelude::{NameOrAddress, TransactionRequest};
use futures::SinkExt;
use primary::{Certificate, Primary};
use store::Store;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::oneshot::{channel as oneshot_channel, Receiver as OneShotReceiver, Sender as OneShotSender};
use worker::Worker;

use rocksdb;
use warp;
use std::collections::HashMap;
use warp::{
    http::Response,
    Filter,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tendermint_abci::ClientBuilder;
use tendermint_proto::abci::{
    RequestDeliverTx,
    RequestQuery,
    ResponseQuery,
    RequestInitChain,
    RequestBeginBlock,
    RequestEndBlock,
    RequestInfo,
};
use tendermint_proto::types::{
    Header,
};


/// The default channel capacity.
pub const CHANNEL_CAPACITY: usize = 1_000;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about("A research implementation of Narwhal and Tusk.")
        .args_from_usage("-v... 'Sets the level of verbosity'")
        .subcommand(
            SubCommand::with_name("generate_keys")
                .about("Print a fresh key pair to file")
                .args_from_usage("--filename=<FILE> 'The file where to print the new key pair'"),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Run a node")
                .args_from_usage("--keys=<FILE> 'The file containing the node keys'")
                .args_from_usage("--committee=<FILE> 'The file containing committee information'")
                .args_from_usage("--parameters=[FILE] 'The file containing the node parameters'")
                .args_from_usage("--store=<PATH> 'The path where to create the data store'")
                .subcommand(
                    SubCommand::with_name("primary")
                        .about("Run a single primary")
                )
                .subcommand(
                    SubCommand::with_name("worker")
                        .about("Run a single worker")
                        .args_from_usage("--id=<INT> 'The worker id'"),
                )
                .setting(AppSettings::SubcommandRequiredElseHelp),
        )
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let log_level = match matches.occurrences_of("v") {
        0 => "error",
        1 => "warn",
        2 => "info",
        3 => "debug",
        _ => "trace",
    };
    let mut logger = env_logger::Builder::from_env(Env::default().default_filter_or(log_level));
    #[cfg(feature = "benchmark")]
    logger.format_timestamp_millis();
    logger.init();

    match matches.subcommand() {
        ("generate_keys", Some(sub_matches)) => KeyPair::new()
            .export(sub_matches.value_of("filename").unwrap())
            .context("Failed to generate key pair")?,
        ("run", Some(sub_matches)) => run(sub_matches).await?,
        _ => unreachable!(),
    }
    Ok(())
}



use warp::{http::StatusCode, reject, Reply, Rejection};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BroadcastTxQuery {
    tx: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AbciQueryQuery {
    path: String,
    data: String,
    height: Option<usize>,
    prove: Option<bool>,
}

// Runs either a worker or a primary.
async fn run(matches: &ArgMatches<'_>) -> Result<()> {
    let key_file = matches.value_of("keys").unwrap();
    let committee_file = matches.value_of("committee").unwrap();
    let parameters_file = matches.value_of("parameters");
    let store_path = matches.value_of("store").unwrap();

    // Read the committee and node's keypair from file.
    let keypair = KeyPair::import(key_file).context("Failed to load the node's keypair")?;
    let committee =
        Committee::import(committee_file).context("Failed to load the committee information")?;

    // Load default parameters if none are specified.
    let parameters = match parameters_file {
        Some(filename) => {
            Parameters::import(filename).context("Failed to load the node's parameters")?
        }
        None => Parameters::default(),
    };

    // Make the data store.
    let store = Store::new(store_path).context("Failed to create a store")?;

    // Channels the sequence of certificates.
    let (tx_output, mut rx_output) = channel(CHANNEL_CAPACITY);

    let name = keypair.name.clone();

    // Check whether to run a primary, a worker, or an entire authority.
    match matches.subcommand() {
        // Spawn the primary and consensus core.
        ("primary", _) => {
            let (tx_new_certificates, rx_new_certificates) = channel(CHANNEL_CAPACITY);
            let (tx_feedback, rx_feedback) = channel(CHANNEL_CAPACITY);

            let keypair_name = keypair.name.clone();

            Primary::spawn(
                keypair,
                committee.clone(),
                parameters.clone(),
                store.clone(),
                /* tx_consensus */ tx_new_certificates,
                /* rx_consensus */ rx_feedback,
            );
            Consensus::spawn(
                committee.clone(),
                parameters.gc_depth,
                /* rx_primary */ rx_new_certificates,
                /* tx_primary */ tx_feedback,
                tx_output,
            );

            // Spawn the network receiver listening to messages from the other primaries.
            let mut app_address = committee
                .primary(&keypair_name.clone())
                .expect("Our public key or worker id is not in the committee")
                .api_abci;
            app_address.set_ip("0.0.0.0".parse().unwrap());

            // address of mempool
            let mempool_address = committee
                .worker(&keypair_name.clone(), &0)
                .expect("Our public key or worker id is not in the committee")
                .transactions;
            
            // ABCI queries will be sent using this from the RPC to the ABCI client
            let (tx_abci_queries, mut rx_abci_queries) = channel(CHANNEL_CAPACITY);


            tokio::spawn(async move {
                // let tx_abci_queries = tx_abci_queries.clone();

                let route_broadcast_tx = warp::path("broadcast_tx")
                    .and(warp::query::<BroadcastTxQuery>())
                    .and_then(move |req: BroadcastTxQuery| async move {
                        log::warn!("broadcast_tx: {:?}", req);

                        let stream = TcpStream::connect(mempool_address)
                            .await
                            .context(format!("ROUTE_BROADCAST_TX failed to connect to {}", mempool_address))
                            .unwrap();
                        let mut transport = Framed::new(stream, LengthDelimitedCodec::new());

                        if let Err(e) = transport.send(req.tx.clone().into()).await {
                            // return Err::<_, Rejection>("Ooops, something went wrong!");
                            Ok::<_, Rejection>(format!("ERROR IN: broadcast_tx: {:?}", req))
                        } else {
                            Ok::<_, Rejection>(format!("broadcast_tx: {:?}", req))
                        }
                    });

                let route_abci_query = warp::path("abci_query")
                    .and(warp::query::<AbciQueryQuery>())
                    .and_then(move |req: AbciQueryQuery| {
                        let tx_abci_queries = tx_abci_queries.clone();
                        async move {
                            log::warn!("abci_query: {:?}", req);

                            let (tx, rx) = oneshot_channel();
                            tx_abci_queries.send((tx, req.clone())).await;
                            let resp = rx.await;

                            Ok::<_, Rejection>(format!("abci_query: {:?} -> {:?}", req, resp))
                        }
                    });

                let route = route_broadcast_tx.or(route_abci_query);

                // Spawn the network receiver listening to messages from the other primaries.
                let mut address = committee
                    .primary(&keypair_name)
                    .expect("Our public key or worker id is not in the committee")
                    .api_rpc;
                address.set_ip("0.0.0.0".parse().unwrap());
                log::warn!(
                    "Primary {} listening to HTTP RPC on {}",
                    keypair_name, address
                );

                warp::serve(route)
                    .run(address).await;
            });


            // Analyze the consensus' output.
            let mut engine = Engine {
                store_path: store_path.to_string(),
                app_address,
                rx_abci_queries,
            };
            engine.run(rx_output).await?;
        }

        // Spawn a single worker.
        ("worker", Some(sub_matches)) => {
            let id = sub_matches
                .value_of("id")
                .unwrap()
                .parse::<WorkerId>()
                .context("The worker id must be a positive integer")?;

            Worker::spawn(
                keypair.name,
                id,
                committee.clone(),
                parameters,
                store.clone(),
            );

            // for a worker there is nothing coming here ...
            rx_output.recv().await;
        }

        _ => unreachable!(),
    }

    // If this expression is reached, the program ends and all other tasks terminate.
    unreachable!();
}

pub struct Engine {
    pub app_address: SocketAddr,
    pub store_path: String,
    pub rx_abci_queries: Receiver<(OneShotSender<String>, AbciQueryQuery)>,
}

/// Receives an ordered list of certificates and apply any application-specific logic.
impl Engine {
    async fn run(&mut self, mut rx_output: Receiver<Certificate>) -> anyhow::Result<()> {
        let mut client = ClientBuilder::default().connect(&self.app_address).unwrap();
        let res = client.info(RequestInfo::default()).unwrap();
        let mut last_block_height = res.last_block_height + 1;
        println!("RequestInfo: {:?}", res);

        let mut client = ClientBuilder::default().connect(&self.app_address).unwrap();

        client.init_chain(RequestInitChain::default()).unwrap();

        loop {
            tokio::select! {

                Some(certificate) = rx_output.recv() => {
                    let mut req = RequestBeginBlock::default();
                    let mut header = Header::default();
                    header.height = last_block_height;
                    last_block_height = last_block_height + 1;
                    req.header = Some(header);
                    match client.begin_block(req.clone()) {
                        Ok(res) => {
                            println!("BeginBlock {:?} -> {:?}", req, res);
                        }
                        Err(err) => log::error!("BeginBlock ERROR {}", err),
                    };

                    for (digest, worker_id) in certificate.header.payload.iter() {
                        // these actually seem to be consistent across honest nodes, according to experiments!
                        log::warn!(
                            "LEDGER {} -> {:?} from {:?}",
                            certificate.header,
                            digest,
                            worker_id
                        );

                        let db = rocksdb::DB::open_for_read_only(
                            &rocksdb::Options::default(),
                            format!("{}-{}", self.store_path, worker_id),
                            true,
                        )?;
                        let key = digest.clone().to_vec();
                        if let Ok(Some(value)) = db.get(&key) {
                            // log::warn!("BATCH {:?}: {:?}", key, value);
                            log::warn!("BATCH FOUND: {:?}", hex::encode(&key));

                            // Deserialize and parse the message.
                            match bincode::deserialize(&value) {
                                Ok(worker::worker::WorkerMessage::Batch(batch)) => {
                                    // log::warn!("BATCH: {:?}", batch);

                                    for tx in batch {
                                        println!("DeliverTx'ing: {:?}", tx);

                                        match client.deliver_tx(RequestDeliverTx { tx: tx.into() }) {
                                            Ok(res) => {
                                                println!("DeliverTx'ed -> {:?}", res);
                                            }
                                            Err(err) => log::error!("DeliverTx ERROR {}", err),
                                        };
                                    }
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            log::error!("BATCH NOT FOUND: {:?}", key);
                            unreachable!()
                        };
                    }

                    let req = RequestEndBlock { height: last_block_height - 1 };
                    match client.end_block(req.clone()) {
                        Ok(res) => {
                            println!("EndBlock {:?} -> {:?}", req, res);
                        }
                        Err(err) => log::error!("EndBlock ERROR {}", err),
                    };

                    match client.commit() {
                        Ok(res) => {
                            println!("Commit -> {:?}", res);
                        }
                        Err(err) => log::error!("Commit ERROR {}", err),
                    };
                },
                Some((tx, req)) = self.rx_abci_queries.recv() => {
                    println!("Query'ing: {:?}", req);

                    let req_height = req.height.unwrap_or(0);
                    let req_prove = req.prove.unwrap_or(false);

                    match client.query(RequestQuery {
                            data: req.data.into(),
                            path: req.path.into(),
                            height: req_height as i64,
                            prove: req_prove,
                        }) {

                        Ok(res) => {
                            println!("Query'ed -> {:?}", res);
                            tx.send(format!("{:?}", res));
                        }
                        Err(err) => log::error!("Query ERROR {}", err),
                    };
                }

                else => break,
            }
        }

        Ok(())
    }
}
