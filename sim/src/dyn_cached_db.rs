use revm::{primitives::{Account, AccountInfo, Address, Bytecode, HashMap, B256, U256}, Database, DatabaseCommit, DatabaseRef};

use crate::storage_provider::StorageProvider;

pub type InnerDb = revm::db::CacheDB<revm::db::EmptyDB>;
pub struct DynamicDatabase<Provider: StorageProvider> {
    inner: InnerDb,
    provider: Provider,
}

impl<Provider: StorageProvider> Database for DynamicDatabase<Provider> {
    type Error = <InnerDb as Database>::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.inner.basic(address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.inner.code_by_hash(code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.inner.storage(address, index)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.inner.block_hash(number)
    }
}

impl<Provider: StorageProvider> DatabaseRef for DynamicDatabase<Provider> {
    type Error = <InnerDb as Database>::Error;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.inner.basic_ref(address)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.inner.code_by_hash_ref(code_hash)
    }

    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.inner.storage_ref(address, index)
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.inner.block_hash_ref(number)
    }
}

impl<Provider: StorageProvider> DatabaseCommit for DynamicDatabase<Provider> {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        self.inner.commit(changes);
    }
}
