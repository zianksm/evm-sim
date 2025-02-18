use std::sync::Arc;

use pyo3::{
    class,
    exceptions::{PyRuntimeError, PyUnboundLocalError},
    FromPyObject, PyErr, PyRef, PyResult,
};
use serde_json::ser;
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

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Clone, Debug)]
pub struct Transaction {
    #[pyo3(get)]
    pub _from: String,
    #[pyo3(get)]
    pub to: String,
    #[pyo3(get)]
    pub value: String,
    #[pyo3(get)]
    pub data: Vec<u8>,
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl Transaction {
    #[new]
    pub fn new(_from: &str, to: &str, value: &str, data: Vec<u8>) -> Self {
        Self {
            _from: _from.to_string(),
            to: to.to_string(),
            value: value.to_string(),
            data,
        }
    }
}

impl TryFrom<Transaction> for Tx {
    type Error = Error;

    fn try_from(v: Transaction) -> Result<Self> {
        let from = Utils::address_try_from_string(v._from)?;

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
#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Clone, Debug)]
pub struct Txs(pub Vec<Tx>);

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl Txs {
    pub fn transactions(&self) -> Vec<Transaction> {
        self.0
            .iter()
            .map(|tx| Transaction {
                _from: tx.from.to_string(),
                to: tx.to.to_string(),
                value: tx.value.to_string(),
                data: tx.data.clone(),
            })
            .collect()
    }

    #[new]
    pub fn new(txs: Vec<Transaction>) -> Self {
        let txs = txs
            .into_iter()
            .map(|tx| Tx::try_from(tx).map_err(|e| e.into()))
            .collect::<Result<Vec<Tx>>>()
            .expect("Failed to serialize transactions");

        Self(txs)
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SimulationResult(ExecutionResult);

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
pub struct Success {
    #[pyo3(get)]
    reason: String,
    #[pyo3(get)]
    gas_used: u64,
    #[pyo3(get)]
    gas_refunded: u64,
    #[pyo3(get)]
    logs: Vec<Log>,
    #[pyo3(get)]
    output: Output,
}

#[derive(Clone, Debug)]
#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
pub struct Log {
    #[pyo3(get)]
    pub address: String,
    #[pyo3(get)]
    pub topics: Vec<Vec<u8>>,
    #[pyo3(get)]
    pub data: Vec<u8>,
}

impl From<evm_primitives::Log> for Log {
    fn from(value: evm_primitives::Log) -> Self {
        Self {
            address: value.address.to_string(),
            topics: value
                .topics()
                .iter()
                .map(|topic| topic.as_slice().to_vec())
                .collect::<Vec<_>>(),
            data: value.data.data.into(),
        }
    }
}

#[derive(Clone, Debug)]
#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
pub struct Output {
    #[pyo3(get)]
    pub is_call: bool,
    #[pyo3(get)]
    pub data: Vec<u8>,
    #[pyo3(get)]
    pub address: Option<String>,
}

impl From<evm_primitives::Output> for Output {
    fn from(value: evm_primitives::Output) -> Self {
        Self {
            // none means it's a contract execution
            is_call: value.address().is_none(),
            data: value.data().to_owned().into(),
            address: value.address().map(|a| a.to_string()),
        }
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl SimulationResult {
    pub fn is_success(&self) -> bool {
        self.0.is_success()
    }

    pub fn is_revert(&self) -> bool {
        std::mem::discriminant(&self.0)
            == std::mem::discriminant(&ExecutionResult::Revert {
                gas_used: 0,
                output: vec![].into(),
            })
    }

    pub fn is_halt(&self) -> bool {
        self.0.is_halt()
    }

    pub fn to_success(&self) -> PyResult<Success> {
        match &self.0 {
            ExecutionResult::Success {
                reason,
                gas_used,
                gas_refunded,
                logs,
                output,
            } => Ok(Success {
                reason: match reason {
                    evm_primitives::SuccessReason::EofReturnContract => {
                        "EofReturnContract".to_string()
                    }
                    evm_primitives::SuccessReason::Stop => "Stop".to_string(),
                    evm_primitives::SuccessReason::Return => "Return".to_string(),
                    evm_primitives::SuccessReason::SelfDestruct => "SelfDestruct".to_string(),
                },
                gas_used: *gas_used,
                gas_refunded: *gas_refunded,
                logs: logs.iter().map(|log| log.to_owned().into()).collect(),
                output: output.clone().into(),
            }),
            _ => Err(PyRuntimeError::new_err("Not a success")),
        }
    }

    pub fn to_revert(&self) -> PyResult<Revert> {
        match &self.0 {
            ExecutionResult::Revert { gas_used, output } => Ok(Revert {
                gas_used: *gas_used,
                output: output.clone().into(),
            }),
            _ => Err(PyRuntimeError::new_err("Not a revert")),
        }
    }
}

#[derive(Clone, Debug)]
#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
pub struct Revert {
    #[pyo3(get)]
    gas_used: u64,
    #[pyo3(get)]
    output: Vec<u8>,
}

impl From<ExecutionResult> for SimulationResult {
    fn from(value: ExecutionResult) -> Self {
        value.into()
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DatabaseErrorRef(String);

impl From<String> for DatabaseErrorRef {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl DatabaseErrorRef {
    pub fn get(&self) -> String {
        self.0.clone()
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EvmExecutionError(ExecutionError<DatabaseErrorRef>);

impl From<ExecutionError> for EvmExecutionError {
    fn from(value: ExecutionError) -> Self {
        let value = match value {
            evm_primitives::EVMError::Transaction(invalid_transaction) => {
                ExecutionError::<DatabaseErrorRef>::Transaction(invalid_transaction)
            }
            evm_primitives::EVMError::Header(invalid_header) => {
                ExecutionError::<DatabaseErrorRef>::Header(invalid_header)
            }

            // we convert them to strings, since we can't pass the error directly(it can but we need to do a whole lot of work)
            evm_primitives::EVMError::Database(e) => {
                ExecutionError::<DatabaseErrorRef>::Database(DatabaseErrorRef(e.to_string()))
            }
            evm_primitives::EVMError::Custom(s) => ExecutionError::<DatabaseErrorRef>::Custom(s),
            evm_primitives::EVMError::Precompile(s) => {
                ExecutionError::<DatabaseErrorRef>::Precompile(s)
            }
        };

        Self(value)
    }
}

pub struct UnderlyingEvmError(String);

impl From<UnderlyingEvmError> for PyErr {
    fn from(value: UnderlyingEvmError) -> Self {
        PyRuntimeError::new_err(value.0)
    }
}

impl From<ExecutionError> for UnderlyingEvmError {
    fn from(value: ExecutionError) -> Self {
        Self(value.to_string())
    }
}

#[pyo3_stub_gen::derive::gen_stub_pyclass]
#[pyo3::pyclass(frozen)]
pub struct EvmSimulator {
    // we wrap it in Arc, since all simulation doesn't require mutable access
    // bonus also to minimize copying data around
    inner: Arc<Simulator>,
}

#[pyo3_stub_gen::derive::gen_stub_pymethods]
#[pyo3::pymethods]
impl EvmSimulator {
    #[new]
    pub fn new(url: &str) -> Result<Self> {
        let inner = Simulator::new(url)?.into();

        Ok(Self { inner })
    }

    #[pyo3(text_signature = "($self, transactions)")]
    pub fn simulate(&self, transactions: Txs) -> PyResult<Vec<SimulationResult>> {
        let txs = transactions.0.clone();
        let result = self.inner.simulate(txs);

        let mut results = vec![];

        for r in result {
            match r {
                Ok(r) => {
                    results.push(r.into());
                }
                Err(e) => {
                    return Err(<UnderlyingEvmError as From<_>>::from(e).into());
                }
            }
        }

        Ok(results)
    }
}
