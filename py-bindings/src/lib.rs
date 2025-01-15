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
use types::{
    DatabaseErrorRef, EvmExecutionError, EvmExecutionResult, EvmSimulator, SimulationResult,
    Transaction, Txs,
};

#[pymodule]
fn evm_sim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Transaction>()?;
    m.add_class::<EvmSimulator>()?;
    m.add_class::<Txs>()?;
    m.add_class::<EvmExecutionResult>()?;
    m.add_class::<SimulationResult>()?;
    m.add_class::<DatabaseErrorRef>()?;
    m.add_class::<EvmExecutionError>()?;

    Ok(())
}
