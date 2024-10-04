use std::convert::Infallible;

use revm::{ *, db::EmptyDBTyped };

pub struct Vm<'a> {
    instance: Evm<'a, (), EmptyDBTyped<Infallible>>,
}

impl Vm<'_> {
    pub fn new() -> Self {
        let vm = EvmBuilder::default().build();
        
        Self {
            instance: vm,
        }
    }

    pub fn execute(&mut self, code: &[u8]) -> Result<(), String> {
        self.instance.transact()
    }
}
