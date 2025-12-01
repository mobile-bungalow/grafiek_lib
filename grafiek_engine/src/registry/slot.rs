use serde::{Deserialize, Serialize};

use crate::ValueType;

/// Metadata for a slot (name, range, UI hints, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlotMetadata {
    pub name: String,
    // TODO: add range, step, UI hints, etc.
}

/// Describes a single slot in a signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDef {
    pub value_type: ValueType,
    pub metadata: SlotMetadata,
}

impl SlotDef {
    pub fn new(value_type: ValueType, metadata: SlotMetadata) -> Self {
        Self {
            value_type,
            metadata,
        }
    }
}
