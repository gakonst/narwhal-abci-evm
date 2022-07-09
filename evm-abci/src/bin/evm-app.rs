use abci::async_api::Server;
use evm_abci::App;
use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(default_value = "0.0.0.0:26658")]
    host: String,
    #[clap(long, short)]
    demo: bool,
}

use tracing_error::ErrorLayer;

use tracing_subscriber::prelude::*;

/// Initializes a tracing Subscriber for logging
#[allow(dead_code)]
pub fn subscriber() {
    tracing_subscriber::Registry::default()
        .with(tracing_subscriber::EnvFilter::new("evm-app=trace"))
        .with(ErrorLayer::default())
        .with(tracing_subscriber::fmt::layer())
        .init()
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    subscriber();

    let App {
        consensus,
        mempool,
        info,
        snapshot,
    } = App::new(args.demo);
    let server = Server::new(consensus, mempool, info, snapshot);

    dbg!(&args.host);
    // let addr = args.host.strip_prefix("http://").unwrap_or(&args.host);
    let addr = args.host.parse::<SocketAddr>().unwrap();

    // let addr = SocketAddr::new(addr, args.port);
    server.run(addr).await?;

    Ok(())
}
