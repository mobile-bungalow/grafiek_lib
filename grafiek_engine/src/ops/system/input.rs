use crate::error::Result;
use crate::traits::{OpPath, Operation, OperationFactory};

pub struct Input;

impl Operation for Input {}

impl OperationFactory for Input {
    const PATH: OpPath = OpPath::new("core", "input");
    const LABEL: &'static str = "Input";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Input))
    }
}
