use crate::OpCategory;
use crate::error::Result;
use crate::traits::{Operation, OperationFactory};

pub struct Input;

impl Operation for Input {}

impl OperationFactory for Input {
    const OPERATION_NAME: &'static str = "Input";
    const LIBRARY_NAME: &'static str = "core";
    const CATEGORY: OpCategory = OpCategory::Engine;

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Input))
    }
}
