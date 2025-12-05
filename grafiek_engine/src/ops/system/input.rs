use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Config, Inputs, Outputs};
use crate::{ConfigSchema, EnumSchema, ExecutionContext};

#[derive(Copy, Clone)]
pub struct Input {
    pub value_type: InputType,
}

#[derive(EnumSchema, Default, Copy, Clone)]
pub enum InputType {
    #[default]
    Float = 0,
    Int,
}

#[derive(ConfigSchema)]
struct InputConfig {
    #[on_node_body]
    #[label("type")]
    value_type: InputType,
}

impl Operation for Input {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(&mut self, _ctx: &mut ExecutionContext, registry: &mut SignatureRegistery) {
        registry.add_output::<f32>("value").build();
        registry.register_config::<InputConfig>();
    }

    fn configure(&mut self, config: Config, registry: &mut SignatureRegistery) -> Result<()> {
        let cfg = InputConfig::try_extract(config)?;
        self.value_type = cfg.value_type;

        registry.clear_outputs();

        match self.value_type {
            InputType::Float => {
                registry.add_output::<f32>("value").build();
            }
            InputType::Int => {
                registry.add_output::<i32>("value").build();
            }
        }

        Ok(())
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        _inputs: Inputs,
        _outputs: Outputs,
    ) -> Result<()> {
        // Output value is set directly via edit_graph_input, nothing to do here
        Ok(())
    }
}

impl OperationFactory for Input {
    const LIBRARY: &'static str = "core";
    const OPERATOR: &'static str = "input";
    const LABEL: &'static str = "Input";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Input {
            value_type: InputType::Float,
        }))
    }
}
