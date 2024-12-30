use alloy::primitives::{TxKind, Uint};
use foundry_evm::{
    backend::{DatabaseError, SharedBackend},
    revm::{
        db::CacheDB,
        primitives::{EVMError, ExecutionResult, SpecId},
        Evm,
    },
    traces::{TracingInspector, TracingInspectorConfig},
};

pub type Address = alloy::primitives::Address;
pub type Uint256 = alloy::primitives::Uint<256, 4>;

#[derive(Debug, Clone)]
pub struct Tx {
    pub from: Address,
    pub to: Address,
    pub value: Uint256,
    pub data: Vec<u8>,
}

pub struct LocalEvm<'a> {
    evm: Evm<'a, TracingInspector, CacheDB<SharedBackend>>,
    txs: Vec<Tx>,
}

impl<'a> LocalEvm<'a> {
    pub fn exec_raw(mut self) -> Vec<Result<ExecutionResult, EVMError<DatabaseError>>> {
        let mut results = Vec::new();

        for tx in self.txs.into_iter() {
            let tx_ref = self.evm.tx_mut();

            tx_ref.clear();
            tx_ref.caller = tx.from;
            tx_ref.gas_limit = u64::MAX;
            tx_ref.transact_to = TxKind::Call(tx.to);
            tx_ref.data = tx.data.into();
            tx_ref.value = tx.value;

            let result = self.evm.transact_commit();

            results.push(result);
        }

        results
    }
}

pub struct EvmFactory {
    pub db: CacheDB<SharedBackend>,
}

impl EvmFactory {
    pub fn with_tx(&self, txs: Vec<Tx>) -> LocalEvm<'_> {
        let context = foundry_evm::revm::Context::new_with_db(self.db.clone());

        let ctx = TracingInspector::new(TracingInspectorConfig::default_geth());

        let evm = Evm::builder()
            .with_db(self.db.clone())
            .with_spec_id(SpecId::CANCUN)
            .modify_cfg_env(|e| {
                e.disable_base_fee = true;
                e.disable_block_gas_limit = true;
                e.limit_contract_code_size = Some(0x100000);
            })
            .with_external_context(ctx)
            .build();

        LocalEvm { evm, txs }
    }
}
