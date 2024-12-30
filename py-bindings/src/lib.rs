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
use types::Transaction;

static mut SIMULATOR: Option<Arc<Simulator>> = None;

#[pyfunction]
pub fn initialize(url: &str) {
    let simulator = Simulator::new(url).unwrap();

    unsafe {
        SIMULATOR = Some(simulator.into());
    }
}

pub fn simulator() -> &'static Arc<Simulator> {
    unsafe { SIMULATOR.as_ref().expect("Simulator not initialized") }
}

#[pyfunction]
pub fn simulate(txs: Vec<Transaction>){
    // simulator().simulate(txs.into());
}

#[pymodule]
fn pyo3_example(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Transaction>()?;
    todo!();
    // m.add_function(wrap_pyfunction!(pyfunction, m)?)
}
