use crate::evm::{EvmFactory, Tx};
use alloy::transports::http::reqwest::Url;
use anyhow::Result;
use foundry_evm::{
    backend::{BlockchainDb, BlockchainDbMeta, DatabaseError, SharedBackend},
    revm::{
        db::CacheDB,
        primitives::{EVMError, ExecutionResult},
    },
};

pub struct Simulator {
    factory: EvmFactory,
}

impl Simulator {
    pub fn new(url: &str) -> Result<Self> {
        let url = Url::parse(url)?;

        let transport = alloy::providers::builder().on_http(url.to_owned());

        let db = BlockchainDb::new(
            BlockchainDbMeta::new(Default::default(), url.to_string()),
            Some("/tmp/evm.db".into()),
        );
        let backend = SharedBackend::spawn_backend_thread(
            transport, db, // we dont want to pin block
            None,
        );

        let db = CacheDB::new(backend);
        let factory = EvmFactory { db };

        Ok(Self { factory })
    }

    pub fn simulate(&self, txs: Vec<Tx>) -> Vec<Result<ExecutionResult, EVMError<DatabaseError>>> {
        let vm = self.factory.with_tx(txs);

        vm.exec_raw()
    }
}
