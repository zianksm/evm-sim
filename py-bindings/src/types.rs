use std::sync::Arc;

use pyo3::{class, FromPyObject, PyRef};
use sim::{
    evm::{Address, Tx, Uint256},
    evm_primitives,
    simulator::{Bytes32, ExecutionError, ExecutionResult, Simulator},
    utils::Utils,
};

pub type Error = GenericError<anyhow::Error>;
pub type Result<T> = std::result::Result<T, GenericError<anyhow::Error>>;

#[derive(Debug, Clone)]
pub struct GenericError<T> {
    inner: T,
}

// bunch of shenanigans to make it work with any errors
impl<T> std::error::Error for GenericError<T> where
    T: AsRef<dyn std::error::Error> + std::fmt::Display + std::fmt::Debug
{
}

impl<T> std::fmt::Display for GenericError<T>
where
    T: AsRef<dyn std::error::Error> + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.inner)
    }
}

impl<T> From<T> for GenericError<T>
where
    T: AsRef<dyn std::error::Error>,
{
    fn from(inner: T) -> Self {
        Self { inner }
    }
}
impl<T> From<GenericError<T>> for pyo3::PyErr
where
    T: AsRef<dyn std::error::Error> + std::fmt::Display + std::fmt::Debug,
{
    fn from(err: GenericError<T>) -> Self {
        pyo3::exceptions::PyException::new_err(err.to_string())
    }
}

#[pyo3::pyclass]
#[derive(Clone, Debug)]
pub struct Transaction {
    from: String,
    to: String,
    value: String,
    data: Vec<u8>,
}

impl TryFrom<Transaction> for Tx {
    type Error = Error;

    fn try_from(v: Transaction) -> Result<Self> {
        let from = Utils::address_try_from_string(v.from)?;

        let to = Utils::address_try_from_string(v.to)?;

        let value = Utils::uint_try_from_string(&v.value)?;

        let data = v.data;

        Ok(Self {
            from,
            to,
            value,
            data,
        })
    }
}

#[derive(Clone, Debug)]
#[pyo3::pyclass]
pub struct Txs(Vec<Tx>);

#[derive(Clone, Debug)]
#[pyo3::pyclass]
pub enum SimulationResult {
    Ok(ExecutionResult),
    Err(ExecutionError<DatabaseErrorRef>),
}

#[derive(Clone, Debug)]
pub struct DatabaseErrorRef(String);

impl From<ExecutionError> for ExecutionError<DatabaseErrorRef> {
    fn from(value: ExecutionError) -> Self {
        match value {
            evm_primitives::EVMError::Transaction(invalid_transaction) => {
                Self::Transaction(invalid_transaction)
            }
            evm_primitives::EVMError::Header(invalid_header) => Self::Header(invalid_header),

            // we convert them to strings, since we can't pass the error directly(it can but we need to do a whole lot of work)
            evm_primitives::EVMError::Database(e) => {
                Self::Database(DatabaseErrorRef(e.to_string()))
            }
            evm_primitives::EVMError::Custom(s) => Self::Custom(s),
            evm_primitives::EVMError::Precompile(s) => Self::Precompile(s),
        }
    }
}

#[pyo3::pyclass(frozen)]
pub struct EvmSimulator {
    // we wrap it in Arc, since all simulation doesn't require mutable access
    // bonus also to minimize copying data around
    inner: Arc<Simulator>,
}

#[pyo3::pymethods]
impl EvmSimulator {
    #[new]
    pub fn new(url: &str) -> Result<Self> {
        let inner = Simulator::new(url)?.into();

        Ok(Self { inner })
    }

    #[pyo3(text_signature = "($self, transactions)")]
    pub fn simulate(&self, transactions: Txs) {
        let txs = transactions.0;
        let result = self.inner.simulate(txs);
        todo!()
    }
}
