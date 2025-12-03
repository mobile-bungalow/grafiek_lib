use parameter_schema_derive::{ConfigSchema, EnumSchema};

#[derive(Default, Clone, EnumSchema)]
enum Enumeration {
    #[default]
    A = -1,
    B = 0,
    C,
}

#[derive(ConfigSchema)]
struct MyConfig {
    field_0: Enumeration,
    threshold: f32,
    enabled: i32,
}

fn main() {
    let _config = MyConfig::default();
}
