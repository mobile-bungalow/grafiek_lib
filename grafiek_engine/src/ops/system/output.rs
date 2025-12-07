use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Inputs, Outputs};
use crate::{CommonMetadata, ExecutionContext};

pub struct Output;

impl Operation for Output {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, _ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        registry.push_input_raw(crate::SlotDef {
            value_type: crate::ValueType::Any,
            name: "value".into(),
            extended: crate::ExtendedMetadata::None,
            common: CommonMetadata::default(),
            default_override: None,
        });
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
