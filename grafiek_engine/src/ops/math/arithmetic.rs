use crate::ConfigSchema;
use crate::EnumSchema;
use crate::ExecutionContext;
use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Config, Inputs, InputsExt, Outputs, OutputsExt};

pub struct Arithmetic {
    pub operation: ArithOp,
}

#[derive(EnumSchema, Default, Copy, Clone)]
pub enum ArithOp {
    #[default]
    Add = 0,
    Subtract,
    Multiply,
    Power,
    Log,
    Divide,
    Min,
    Max,
    Abs,
}

#[derive(ConfigSchema)]
struct AddConfig {
    operation: ArithOp,
}

impl Operation for Arithmetic {
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
        registry.register_config::<AddConfig>();
    }

    fn configure(&mut self, config: Config, registry: &mut SignatureRegistery) -> Result<()> {
        registry.clear();
        registry.add_output::<f32>("result").build();
        let cfg = AddConfig::try_extract(config)?;

        self.operation = cfg.operation;

        match cfg.operation {
            ArithOp::Add | ArithOp::Multiply | ArithOp::Max | ArithOp::Min => {
                registry.add_input::<f32>("a").build();
                registry.add_input::<f32>("b").build();
            }
            ArithOp::Subtract => {
                registry.add_input::<f32>("minuend").build();
                registry.add_input::<f32>("subtrahend").build();
            }
            ArithOp::Power => {
                registry.add_input::<f32>("base").build();
                registry.add_input::<f32>("exponent").build();
            }
            ArithOp::Log => {
                registry.add_input::<f32>("base").build();
                registry.add_input::<f32>("a").build();
            }
            ArithOp::Divide => {
                registry.add_input::<f32>("dividend").build();
                registry.add_input::<f32>("divisor").build();
            }
            ArithOp::Abs => {
                registry.add_input::<f32>("a").build();
            }
        }

        Ok(())
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<()> {
        match self.operation {
            ArithOp::Add => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a + b;
            }
            ArithOp::Subtract => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a - b;
            }
            ArithOp::Multiply => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a * b;
            }
            ArithOp::Power => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a.powf(b);
            }
            ArithOp::Log => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a.log(b);
            }
            ArithOp::Divide => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a / b;
            }
            ArithOp::Min => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a.min(b);
            }
            ArithOp::Max => {
                let a: f32 = inputs.extract(0)?;
                let b: f32 = inputs.extract(1)?;
                *outputs.extract::<f32>(0)? = a.max(b);
            }
            ArithOp::Abs => {
                let a: f32 = inputs.extract(0)?;
                *outputs.extract::<f32>(0)? = a.abs()
            }
        }

        Ok(())
    }
}

impl OperationFactory for Arithmetic {
    const LIBRARY: &'static str = "math";
    const OPERATOR: &'static str = "add";
    const LABEL: &'static str = "Add";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Arithmetic {
            operation: ArithOp::Add,
        }))
    }
}
