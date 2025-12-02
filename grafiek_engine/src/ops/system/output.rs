use std::any::Any;

use crate::ExecutionContext;
use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, Outputs};

pub struct Output;

impl Operation for Output {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, _ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        registry.add_input::<f32>("value").build();
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        _inputs: Inputs,
        _outputs: Outputs,
    ) -> Result<()> {
        Ok(())
    }
}

impl OperationFactory for Output {
    const LIBRARY: &'static str = "core";
    const OPERATOR: &'static str = "output";
    const LABEL: &'static str = "Output";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Output))
    }
}
