use crate::{Consensus, Info, Mempool, Snapshot, State};
use foundry_evm::revm::{
    db::{CacheDB, EmptyDB},
    AccountInfo,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct App<Db> {
    pub mempool: Mempool,
    pub snapshot: Snapshot,
    pub consensus: Consensus<Db>,
    pub info: Info<Db>,
}

impl Default for App<CacheDB<EmptyDB>> {
    fn default() -> Self {
        Self::new(false)
    }
}

impl App<CacheDB<EmptyDB>> {
    pub fn new(demo: bool) -> Self {
        let mut state = State {
            db: CacheDB::new(EmptyDB()),
            block_height: Default::default(),
            app_hash: Default::default(),
            env: Default::default(),
        };

        if demo {
            // addr(pk = 78aaa1de82137f31ac551fd8e876a6930aadd51b28c25e8c3420100f8e51d5c6)
            state.db.insert_account_info(
                "0xAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                    .parse()
                    .unwrap(),
                AccountInfo {
                    balance: ethers::utils::parse_ether(1.5).unwrap(),
                    ..Default::default()
                },
            );
        }

        let committed_state = Arc::new(Mutex::new(state.clone()));
        let current_state = Arc::new(Mutex::new(state));

        let consensus = Consensus {
            committed_state: committed_state.clone(),
            current_state,
        };
        let mempool = Mempool::default();
        let info = Info {
            state: committed_state,
        };
        let snapshot = Snapshot::default();

        App {
            consensus,
            mempool,
            info,
            snapshot,
        }
    }
}
