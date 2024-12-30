use sim::evm::{Address, Tx};

#[pyo3::pyclass]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub value: String,
    pub data: Vec<u8>,
}

impl TryFrom<Transaction> for Tx {
    type Error = anyhow::Error;

    fn try_from(value: Transaction) -> Result<Self, Self::Error> {
        todo!()
    }
}
