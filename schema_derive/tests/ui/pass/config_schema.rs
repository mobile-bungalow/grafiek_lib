use parameter_schema_derive::ConfigSchema;

#[derive(ConfigSchema)]
struct MyConfig {
    threshold: f32,
    enabled: i32,
}

fn main() {
    let _config = MyConfig::default();
}
