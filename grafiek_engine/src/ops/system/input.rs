use crate::error::Result;
use crate::registry::{SignatureRegistery, TextureMeta};
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::value::{Config, Inputs, Outputs, OutputsExt};
use crate::{ConfigSchema, EnumSchema, ExecutionContext, SPECK, TextureHandle, Value};

#[derive(Clone)]
pub struct Input {
    pub value_type: InputType,
    value: Value,
}

impl Input {
    pub fn new(value_type: InputType) -> Self {
        let value = match value_type {
            InputType::Float => Value::F32(0.0),
            InputType::Int => Value::I32(0),
            InputType::Texture => Value::Texture(SPECK),
        };
        Self { value_type, value }
    }

    /// Get a reference to the stored value
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Get a mutable reference to the stored value
    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.value
    }

    /// Set the stored value
    pub fn set_value(&mut self, value: Value) {
        self.value = value;
    }
}

#[derive(EnumSchema, Default, Copy, Clone, PartialEq)]
pub enum InputType {
    #[default]
    Float = 0,
    Int,
    Texture,
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

    fn configure(
        &mut self,
        _ctx: &ExecutionContext,
        config: Config,
        registry: &mut SignatureRegistery,
    ) -> Result<()> {
        let cfg = InputConfig::try_extract(config)?;
        let old_type = self.value_type;
        self.value_type = cfg.value_type;

        registry.clear_outputs();

        match self.value_type {
            InputType::Float => {
                registry.add_output::<f32>("value").build();
                // Reset value if type changed
                if old_type != InputType::Float {
                    self.value = Value::F32(0.0);
                }
            }
            InputType::Int => {
                registry.add_output::<i32>("value").build();
                if old_type != InputType::Int {
                    self.value = Value::I32(0);
                }
            }
            InputType::Texture => {
                registry
                    .add_output::<TextureHandle>("value")
                    .default(SPECK)
                    .meta(TextureMeta {
                        preview: true,
                        allow_file: true,
                    })
                    .build();
            }
        }

        Ok(())
    }

    fn execute(
        &mut self,
        _ctx: &mut ExecutionContext,
        _inputs: Inputs,
        mut outputs: Outputs,
    ) -> Result<()> {
        // Write stored value to output
        match &self.value {
            Value::F32(v) => *outputs.extract::<f32>(0)? = *v,
            Value::I32(v) => *outputs.extract::<i32>(0)? = *v,
            Value::Texture(v) => *outputs.extract::<TextureHandle>(0)? = v.clone(),
            _ => {}
        }
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
            value: Value::F32(0.0),
        }))
    }
}
