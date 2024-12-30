use std::sync::Arc;

use pyo3::{class, FromPyObject, PyRef};
use sim::{
    evm::{Address, Tx, Uint256},
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

#[pyo3::pymethods]
impl Transaction {
    #[new]
    pub fn new(from: String, to: String, value: String, data: Vec<u8>) -> Self {
        Self {
            from,
            to,
            value,
            data,
        }
    }

    #[getter]
    pub fn from(&self) -> String {
        self.from.clone()
    }

    #[getter]
    pub fn to(&self) -> String {
        self.to.clone()
    }

    #[getter]
    pub fn value(&self) -> String {
        self.value.clone()
    }

    #[getter]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    #[setter]
    pub fn set_from(&mut self, from: String) {
        self.from = from;
    }

    #[setter]
    pub fn set_to(&mut self, to: String) {
        self.to = to;
    }

    #[setter]
    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }

    #[setter]
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

#[derive(Debug, Clone)]
#[pyo3::pyclass]
pub struct Success {
    reason: SuccessReason,
    gas_used: u64,
    gas_refunded: u64,
    output: Vec<u8>,
    logs: Vec<EvmLog>,
}

#[derive(Debug, Clone)]
#[pyo3::pyclass]
pub struct EvmLog {
    address: String,
    data: LogData,
}

#[derive(Debug, Clone)]
#[pyo3::pyclass]
pub struct LogData {
    topics: Vec<Bytes32>,
    data: Vec<u8>,
}

#[derive(Debug, Clone)]
#[pyo3::pyclass]
pub enum SuccessReason {
    Stop,
    Return,
    SelfDestruct,
    EofReturnContract,
}

impl std::fmt::Display for SuccessReason {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Stop => write!(f, "Stop"),
            Self::Return => write!(f, "Return"),
            Self::SelfDestruct => write!(f, "SelfDestruct"),
            Self::EofReturnContract => write!(f, "EofReturnContract"),
        }
    }
}

#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub struct Revert {
    gas_used: u64,
    output: Vec<u8>,
}

#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub struct Halt {
    gas_used: u64,
    reason: HaltReason,
}

#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub enum OutOfGasError {
    // Basic OOG error
    Basic,
    // Tried to expand past REVM limit
    MemoryLimit,
    // Basic OOG error from memory expansion
    Memory,
    // Precompile threw OOG error
    Precompile,
    // When performing something that takes a U256 and casts down to a u64, if its too large this would fire
    // i.e. in `as_usize_or_fail`
    InvalidOperand,
}

#[pyo3::pyclass]
#[derive(Debug, Clone)]
pub enum HaltReason {
    OpcodeNotFound(),
    InvalidFEOpcode(),
    InvalidJump(),
    NotActivated(),
    StackUnderflow(),
    StackOverflow(),
    OutOfOffset(),
    CreateCollision(),
    PrecompileError(),
    NonceOverflow(),
    /// Create init code size exceeds limit (runtime).
    CreateContractSizeLimit(),
    /// Error on created contract that begins with EF
    CreateContractStartingWithEF(),
    /// EIP-3860: Limit and meter initcode. Initcode size limit exceeded.
    CreateInitCodeSizeLimit(),

    /* Internal Halts that can be only found inside Inspector */
    OverflowPayment(),
    StateChangeDuringStaticCall(),
    CallNotAllowedInsideStatic(),
    OutOfFunds(),
    CallTooDeep(),

    /// Aux data overflow(), new aux data is larger than u16 max size.
    EofAuxDataOverflow(),
    /// Aud data is smaller then already present data size.
    EofAuxDataTooSmall(),
    /// EOF Subroutine stack overflow
    EOFFunctionStackOverflow(),
    /// Check for target address validity is only done inside subcall.
    InvalidEXTCALLTarget(),

    /* Optimism errors */
    FailedDeposit(),

    OutOfGas(OutOfGasError),
}

#[derive(Debug)]
#[pyo3::pyclass]
pub enum SimulationResult {
    Success(Success),
    Revert(Revert),
    Halt(Halt),
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
    pub fn simulate(&self, transactions: Vec<Transaction>) {}
}
