use crate::error::Result;
use crate::registry::SignatureRegistery;
use crate::traits::{OpPath, Operation, OperationFactory};
use crate::{ExecutionContext, TextureHandle};

pub struct Input;

// TODO: derive schema
//#[derive(Schema)]
pub enum InputType {
    Float,
    Integer,
    Texture,
}

//#[derive(Schema(Config))]
pub struct Cfg {
    //#[label("type")]
    ty: InputType,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            ty: InputType::Float,
        }
    }
}

impl Operation for Input {
    fn is_stateful(&self) -> bool {
        false
    }

    fn op_path(&self) -> OpPath {
        <Self as OperationFactory>::op_path()
    }

    fn setup(
        &mut self,
        _ctx: &mut ExecutionContext,
        registry: &mut SignatureRegistery,
    ) -> Result<()> {
        registry.register_config::<Cfg>()?;
        registry.push_output::<f32>("value").finish();
        Ok(())
    }

    fn configure(&mut self, config: Config, registry: &mut SignatureRegistry) -> Result<()> {
        registry.clear();
        let cfg = config.extract::<Cfg>()?;

        match cfg.ty {
            InputType::Float => {
                registry.push_output::<f32>("value").finish()?;
            }
            InputType::Integer => {
                registry.push_output::<i32>("value").finish()?;
            }
            InputType::Texture => {
                registry.push_output::<TextureHandle>("value").finish()?;
            }
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
