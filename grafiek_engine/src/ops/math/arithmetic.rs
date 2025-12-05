use crate::ConfigSchema;
use crate::EnumSchema;
use crate::ExecutionContext;
use crate::error::Result;
use crate::registry::{FloatRange, SignatureRegistery};
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Config, Inputs, InputsExt, Outputs, OutputsExt};

const F32_META: FloatRange = FloatRange {
    min: f32::MIN,
    max: f32::MAX,
    step: 0.1,
    default: 0.0,
};

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
    #[label("")]
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
        registry.add_input::<f32>("a").meta(F32_META).build();
        registry.add_input::<f32>("b").meta(F32_META).build();
        registry.add_output::<f32>("result").build();
        registry.register_config::<AddConfig>();
    }

    fn configure(&mut self, config: Config, registry: &mut SignatureRegistery) -> Result<()> {
        let cfg = AddConfig::try_extract(config)?;
        self.operation = cfg.operation;

        registry.clear_inputs();

        match cfg.operation {
            ArithOp::Add | ArithOp::Multiply | ArithOp::Max | ArithOp::Min => {
                registry.add_input::<f32>("a").meta(F32_META).build();
                registry.add_input::<f32>("b").meta(F32_META).build();
            }
            ArithOp::Subtract => {
                registry.add_input::<f32>("minuend").meta(F32_META).build();
                registry
                    .add_input::<f32>("subtrahend")
                    .meta(F32_META)
                    .build();
            }
            ArithOp::Power => {
                registry.add_input::<f32>("base").meta(F32_META).build();
                registry.add_input::<f32>("exponent").meta(F32_META).build();
            }
            ArithOp::Log => {
                registry.add_input::<f32>("base").meta(F32_META).build();
                registry.add_input::<f32>("a").meta(F32_META).build();
            }
            ArithOp::Divide => {
                registry.add_input::<f32>("dividend").meta(F32_META).build();
                registry.add_input::<f32>("divisor").meta(F32_META).build();
            }
            ArithOp::Abs => {
                registry.add_input::<f32>("a").meta(F32_META).build();
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
        let a: f32 = inputs.extract(0)?;
        let b: f32 = inputs.extract(1).unwrap_or(0.);
        match self.operation {
            ArithOp::Add => {
                *outputs.extract::<f32>(0)? = a + b;
            }
            ArithOp::Subtract => {
                *outputs.extract::<f32>(0)? = a - b;
            }
            ArithOp::Multiply => {
                *outputs.extract::<f32>(0)? = a * b;
            }
            ArithOp::Power => {
                *outputs.extract::<f32>(0)? = a.powf(b);
            }
            ArithOp::Log => {
                *outputs.extract::<f32>(0)? = a.log(b);
            }
            ArithOp::Divide => {
                *outputs.extract::<f32>(0)? = a / b;
            }
            ArithOp::Min => {
                *outputs.extract::<f32>(0)? = a.min(b);
            }
            ArithOp::Max => {
                *outputs.extract::<f32>(0)? = a.max(b);
            }
            ArithOp::Abs => *outputs.extract::<f32>(0)? = a.abs(),
        }

        Ok(())
    }
}

impl OperationFactory for Arithmetic {
    const LIBRARY: &'static str = "math";
    const OPERATOR: &'static str = "arithmetic";
    const LABEL: &'static str = "Arithmetic";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Arithmetic {
            operation: ArithOp::Add,
        }))
    }
}
