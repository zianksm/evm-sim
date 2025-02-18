use std::sync::Arc;

pub mod types;

use pyo3::{
    pyfunction, pymodule,
    types::{PyModule, PyModuleMethods},
    wrap_pyfunction, Bound, PyResult,
};
use pyo3_stub_gen::define_stub_info_gatherer;
use sim::{
    evm::Tx,
    simulator::{ExecutionError, ExecutionResult, Simulator},
};
use types::{
    DatabaseErrorRef, EvmExecutionError, EvmSimulator, Log, Output, SimulationResult, Success,
    Transaction, Txs,
};

#[pymodule]
fn evm_sim(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Transaction>()?;
    m.add_class::<EvmSimulator>()?;
    m.add_class::<Txs>()?;
    m.add_class::<SimulationResult>()?;
    m.add_class::<DatabaseErrorRef>()?;
    m.add_class::<EvmExecutionError>()?;
    m.add_class::<Success>()?;
    m.add_class::<Log>()?;
    m.add_class::<Output>()?;

    Ok(())
}

// Define a function to gather stub information.
define_stub_info_gatherer!(stub_info);
