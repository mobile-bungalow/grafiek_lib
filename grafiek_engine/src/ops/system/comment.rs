use crate::error::Result;
use crate::traits::{OpPath, Operation, OperationFactory};

/// A visual note. Left up to the client to
/// decide display logic.
pub struct Comment;

impl Operation for Comment {}

impl OperationFactory for Comment {
    const PATH: OpPath = OpPath::new("core", "comment");
    const LABEL: &'static str = "Comment";

    fn build() -> Result<Box<dyn Operation>> {
        Ok(Box::new(Comment))
    }
}
