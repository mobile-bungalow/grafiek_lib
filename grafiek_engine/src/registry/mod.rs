mod signature;
mod slot;

pub use signature::*;
pub use slot::*;

/// Placeholder for slot metadata hints (range, step, UI hints, etc.)
/// This is separate from SlotMetadata in value.rs which is per-slot runtime data.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    // TODO: add field-level hints like:
    // - min/max range
    // - step size
    // - display format
    // - visibility conditions
}
