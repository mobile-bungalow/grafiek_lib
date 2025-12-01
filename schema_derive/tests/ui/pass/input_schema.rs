use parameter_schema_derive::InputSchema;

#[derive(InputSchemaEnum)]
enum Options {
    Nearest,
    Linear,
}

#[derive(InputSchema)]
struct MyInputs {
    #[label("Value")]
    #[param(min = 0.0, max = 20.0, step = 0.1, default = 10.0)]
    value: f32,
    count: i32,
    options: Options,
}

fn main() {
    let _inputs = MyInputs::default();
}
