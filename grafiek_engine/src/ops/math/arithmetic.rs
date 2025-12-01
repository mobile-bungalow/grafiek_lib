use crate::error::Result;
use crate::registry::{SignatureRegistery, SlotMetadata};
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, Outputs, ValueRef};
use crate::{ExecutionContext, ValueType};

pub struct Add;

impl Operation for Add {
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
                name: "a".to_string(),
            },
        );
        registry.add_input(
            ValueType::F32,
            SlotMetadata {
                name: "b".to_string(),
            },
        );
        registry.add_output(
            ValueType::F32,
            SlotMetadata {
                name: "result".to_string(),
            },
        );
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<()> {
        let a = match &inputs[0] {
            ValueRef::F32(v) => **v,
            _ => 0.0,
        };
        let b = match &inputs[1] {
            ValueRef::F32(v) => **v,
            _ => 0.0,
        };

        match &mut outputs[0] {
            crate::ValueMut::F32(v) => {
                **v = a + b;
            }
            _ => {}
        };

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
