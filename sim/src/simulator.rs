use crate::evm::{EvmFactory, Tx};
use alloy::transports::{http::reqwest::Url, Transport};
use anyhow::Result;
use foundry_evm::{
    backend::{BlockchainDb, BlockchainDbMeta, DatabaseError, SharedBackend},
    revm::db::CacheDB,
};

pub type ExecutionResult = foundry_evm::revm::primitives::ExecutionResult;
pub type ExecutionError<T = DatabaseError> = foundry_evm::revm::primitives::EVMError<T>;
pub type Bytes32 = foundry_evm::revm::primitives::FixedBytes<32>;

pub struct Simulator {
    factory: EvmFactory,
}

impl Simulator {
    pub fn new(url: &str) -> Result<Self> {
        let url = Url::parse(url)?;

        let db = BlockchainDb::new(
            BlockchainDbMeta::new(Default::default(), url.to_string()),
            Some("/tmp/evm.db".into()),
        );

        let backend = match url.scheme() {
            "http" => {
                let transport = alloy::providers::builder().on_http(url);
                SharedBackend::spawn_backend_thread(
                    transport, db, // we dont want to pin block
                    None,
                )
            }

            "ws" => {
                let transport = smol::block_on(async {
                    alloy::providers::builder()
                        .on_ws(alloy::providers::WsConnect::new(url))
                        .await
                })
                .expect("Failed to connect to websocket");

                SharedBackend::spawn_backend_thread(
                    transport, db, // we dont want to pin block
                    None,
                )
            }
            _ => return Err(anyhow::anyhow!("Unsupported protocol")),
        };

        let db = CacheDB::new(backend);
        let factory = EvmFactory { db };

        Ok(Self { factory })
    }

    pub fn simulate(&self, txs: Vec<Tx>) -> Vec<Result<ExecutionResult, ExecutionError>> {
        let vm = self.factory.with_tx(txs);

        vm.exec_raw()
    }
}
