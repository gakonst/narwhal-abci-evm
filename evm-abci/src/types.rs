use ethers::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;

use abci::{
    async_api::{
        Consensus as ConsensusTrait, Info as InfoTrait, Mempool as MempoolTrait,
        Snapshot as SnapshotTrait,
    },
    async_trait,
    types::*,
};

use foundry_evm::revm::{self, CreateScheme, Database, DatabaseCommit, Env, TransactTo, TxEnv};

/// The app's state, containing a Revm DB.
// TODO: Should we instead try to replace this with Anvil and implement traits for it?
#[derive(Clone, Debug)]
pub struct State<Db> {
    pub block_height: i64,
    pub app_hash: Vec<u8>,
    pub db: Db,
    pub env: Env,
}

impl<Db: Database + DatabaseCommit> State<Db> {
    async fn execute(&mut self, tx: TransactionRequest, read_only: bool) -> eyre::Result<()> {
        let mut evm = revm::EVM::new();
        evm.env = self.env.clone();
        evm.env.tx = TxEnv {
            caller: tx.from.unwrap_or_default(),
            transact_to: match tx.to {
                Some(NameOrAddress::Address(inner)) => TransactTo::Call(inner),
                Some(NameOrAddress::Name(_)) => panic!("not allowed"),
                None => TransactTo::Create(CreateScheme::Create),
            },
            data: tx.data.unwrap_or_default().0,
            chain_id: Some(self.env.cfg.chain_id.as_u64()),
            nonce: Some(tx.nonce.unwrap_or_default().as_u64()),
            value: tx.value.unwrap_or_default(),
            gas_price: tx.gas_price.unwrap_or_default(),
            gas_priority_fee: tx.gas,
            gas_limit: tx.gas.unwrap_or_default().as_u64(),
            access_list: vec![],
        };
        evm.database(&mut self.db);

        let (_ret, _tx_out, _gas, state, _logs) = evm.transact();
        if !read_only {
            self.db.commit(state);
        };

        Ok(())
    }
}

pub struct Consensus<Db> {
    pub committed_state: Arc<Mutex<State<Db>>>,
    pub current_state: Arc<Mutex<State<Db>>>,
}

#[async_trait]
impl<Db: Clone + Send + Sync + DatabaseCommit + Database> ConsensusTrait for Consensus<Db> {
    async fn init_chain(&self, _init_chain_request: RequestInitChain) -> ResponseInitChain {
        ResponseInitChain::default()
    }

    async fn begin_block(&self, _begin_block_request: RequestBeginBlock) -> ResponseBeginBlock {
        ResponseBeginBlock::default()
    }

    async fn deliver_tx(&self, deliver_tx_request: RequestDeliverTx) -> ResponseDeliverTx {
        let mut state = self.current_state.lock().await;

        let mut tx: TransactionRequest = match serde_json::from_slice(&deliver_tx_request.tx) {
            Ok(tx) => tx,
            // no-op just logger
            Err(_) => {
                return ResponseDeliverTx {
                    data: "could not decode request".into(),
                    ..Default::default()
                }
            }
        };

        // resolve the `to`
        match tx.to {
            Some(NameOrAddress::Address(addr)) => tx.to = Some(addr.into()),
            _ => panic!("not an address"),
        };

        let _result = state.execute(tx, false).await.unwrap();

        ResponseDeliverTx::default()
    }

    async fn end_block(&self, end_block_request: RequestEndBlock) -> ResponseEndBlock {
        println!("END BLOCK");
        let mut current_state = self.current_state.lock().await;

        current_state.block_height = end_block_request.height;
        current_state.app_hash = vec![];

        ResponseEndBlock::default()
    }

    async fn commit(&self, _commit_request: RequestCommit) -> ResponseCommit {
        println!("COMMIT");
        let current_state = self.current_state.lock().await.clone();
        let mut committed_state = self.committed_state.lock().await;
        *committed_state = current_state;

        ResponseCommit {
            data: vec![], // (*committed_state).app_hash.clone(),
            retain_height: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Mempool;

#[async_trait]
impl MempoolTrait for Mempool {
    async fn check_tx(&self, _check_tx_request: RequestCheckTx) -> ResponseCheckTx {
        ResponseCheckTx::default()
    }
}

#[derive(Debug, Clone)]
pub struct Info<Db> {
    pub state: Arc<Mutex<State<Db>>>,
}

#[async_trait]
impl<Db: Send + Sync + Database + DatabaseCommit> InfoTrait for Info<Db> {
    // replicate the eth_call interface
    async fn query(&self, query_request: RequestQuery) -> ResponseQuery {
        let mut state = self.state.lock().await;

        let mut tx: TransactionRequest = match serde_json::from_slice(&query_request.data) {
            Ok(tx) => tx,
            // no-op just logger
            Err(_) => {
                return ResponseQuery {
                    value: "could not decode request".into(),
                    ..Default::default()
                }
            }
        };

        match tx.to {
            Some(NameOrAddress::Address(addr)) => tx.to = Some(addr.into()),
            _ => panic!("not an address"),
        };

        let _result = state.execute(tx, true).await.unwrap();

        ResponseQuery {
            key: query_request.data,
            value: vec![],
            ..Default::default()
        }
    }

    async fn info(&self, _info_request: RequestInfo) -> ResponseInfo {
        let state = self.state.lock().await;

        ResponseInfo {
            data: Default::default(),
            version: Default::default(),
            app_version: Default::default(),
            last_block_height: (*state).block_height,
            last_block_app_hash: (*state).app_hash.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot;

impl SnapshotTrait for Snapshot {}
