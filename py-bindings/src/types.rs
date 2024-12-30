use std::{result::Result, sync::Arc};

use pyo3::FromPyObject;
use sim::{
    evm::{Address, Tx, Uint256},
    simulator::Simulator,
    utils::Utils,
};

pub type Result<T> = std::result::Result<T, GenericError<anyhow::Error>>;

#[derive(Debug, Clone)]
pub struct GenericError<T> {
    inner: T,
}

impl<T: std::error::Error> std::error::Error for GenericError<T> {}

impl<T: std::error::Error> std::fmt::Display for GenericError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.inner)
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
    type Error = anyhow::Error;

    fn try_from(v: Transaction) -> Result<Self, Self::Error> {
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

#[derive(Clone)]
#[pyo3::pyclass]
pub struct EvmSimulator {
    // we wrap it in Arc, since all simulation doesn't require mutable access
    // pyo3 require the struct to be cloneable but since we don't actually have to
    // we can just wrap it in Arc to minimize copying data around
    inner: Arc<Simulator>,
}

#[pyo3::pymethods]
impl EvmSimulator {
    #[new]
    pub fn new(url: &str) -> Result<Self> {
        let inner = Simulator::new(url)?.into();

        Ok(Self { inner })
    }
}
