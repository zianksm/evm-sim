use pyo3::FromPyObject;
use sim::{
    evm::{Address, Tx, Uint256},
    utils::Utils,
};

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
