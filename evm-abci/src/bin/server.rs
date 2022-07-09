use abci::async_api::Server;
use evm_abci::App;
use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(default_value = "0.0.0.0:26658")]
    host: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let App {
        consensus,
        mempool,
        info,
        snapshot,
    } = App::new();
    let server = Server::new(consensus, mempool, info, snapshot);

    dbg!(&args.host);
    // let addr = args.host.strip_prefix("http://").unwrap_or(&args.host);
    let addr = args.host.parse::<SocketAddr>().unwrap();

    // let addr = SocketAddr::new(addr, args.port);
    server.run(addr).await?;

    Ok(())
}
