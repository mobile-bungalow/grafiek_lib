use crate::error::Result;
use crate::registry::{SignatureRegistery, SlotMetadata};
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, Outputs};
use crate::{ExecutionContext, ValueType};

pub struct Output;

impl Operation for Output {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, _ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        registry.add_input(
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
        // Output nodes just hold the value for get_graph_output to read
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
