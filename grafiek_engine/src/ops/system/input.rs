use crate::error::Result;
use crate::registry::{SignatureRegistery, SlotMetadata};
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, Outputs};
use crate::{ExecutionContext, ValueType};

pub struct Input;

impl Operation for Input {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, _ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        // Input node stores a value and forwards it to its output
        registry.add_input(
            ValueType::F32,
            SlotMetadata {
                name: "value".to_string(),
            },
        );
        registry.add_output(
            ValueType::F32,
            SlotMetadata {
                name: "value".to_string(),
            },
        );
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        _inputs: Inputs,
        _outputs: Outputs,
    ) -> Result<()> {
        // Forward input to output
        // TODO: copy input[0] to output[0]
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
