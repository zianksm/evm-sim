use revm::{ db::EmptyDB, primitives::{ Address, HashMap, U256 }, Inspector };

pub struct StorageObserver {
    observed_keys: HashMap<Address, HashMap<U256, U256>>,
    observed_results: HashMap<Address, Vec<ObservedResult>>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ObservedResult {
    address: Address,
    key: U256,
    new_value: U256,
    old_value: U256,
}

pub type DefaultDb = revm::db::CacheDB<EmptyDB>;

impl Inspector<DefaultDb> for StorageObserver {
    fn call_end(
        &mut self,
        context: &mut revm::EvmContext<DefaultDb>,
        inputs: &revm::interpreter::CallInputs,
        outcome: revm::interpreter::CallOutcome
    ) -> revm::interpreter::CallOutcome {
        let mut result = Vec::new();

        for entry in context.journaled_state.journal.clone().iter().flatten() {
            let revm::JournalEntry::StorageChanged { address, key, had_value } = entry else {
                continue;
            };

            let new_value = context
                .sload(address.clone(), key.clone())
                .expect("Failed to load storage value");

            let res = ObservedResult {
                address: address.clone(),
                key: key.clone(),
                new_value: new_value.data.clone(),
                old_value: had_value.clone(),
            };
        }

        result
            .into_iter()
            .for_each(|result: ObservedResult|
                self.observed_results
                    .entry(result.address.clone())
                    .or_insert(Vec::new())
                    .push(result)
            );

        outcome
    }
}
