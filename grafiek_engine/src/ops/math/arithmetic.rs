use crate::ExecutionContext;
use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, InputsExt, Outputs, OutputsExt};

pub struct Add;

impl Operation for Add {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, _ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        registry.add_input::<f32>("a").build();
        registry.add_input::<f32>("b").build();
        registry.add_output::<f32>("result").build();
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<()> {
        let a: f32 = inputs.extract(0)?;
        let b: f32 = inputs.extract(1)?;
        *outputs.extract::<f32>(0)? = a + b;
        Ok(())
    }
}

impl OperationFactory for Add {
    const LIBRARY: &'static str = "math";
    const OPERATOR: &'static str = "add";
    const LABEL: &'static str = "Add";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Add))
    }
}
