use crate::ExecutionContext;
use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, Outputs};

pub struct Input;

impl Operation for Input {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, _ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        registry.add_input::<f32>("value").build();
        registry.add_output::<f32>("value").build();
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<()> {
        if let (crate::ValueRef::F32(input), crate::ValueMut::F32(output)) =
            (&inputs[0], &mut outputs[0])
        {
            **output = **input;
        }
        Ok(())
    }
}

impl OperationFactory for Input {
    const LIBRARY: &'static str = "core";
    const OPERATOR: &'static str = "input";
    const LABEL: &'static str = "Input";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Input))
    }
}
