use std::sync::Arc;

pub mod types;

use pyo3::{
    pyfunction, pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction, Bound, PyResult,
};
use sim::{
    evm::Tx,
    simulator::{ExecutionError, ExecutionResult, Simulator},
};
use types::{EvmSimulator, Transaction};

#[pymodule]
fn pyo3_example(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Transaction>()?;
    m.add_class::<EvmSimulator>()?;

    Ok(())
}
