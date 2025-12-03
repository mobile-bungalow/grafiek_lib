use grafiek_engine::{FloatRange, IntRange};
use parameter_schema_derive::ConfigSchema;

#[derive(ConfigSchema)]
struct BlurConfig {
    #[meta(FloatRange { min: 0.0, max: 100.0, ..Default::default() })]
    radius: f32,

    #[meta(IntRange { min: 1, max: 10, ..Default::default() })]
    iterations: i32,

    // No meta attribute - uses simple SlotDef::new
    enabled: i32,
}

fn main() {
    let _config = BlurConfig::default();
}
