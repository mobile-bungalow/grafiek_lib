use parameter_schema_derive::OutputSchema;

#[derive(OutputSchema)]
struct MyOutputs {
    result: f32,
}

fn main() {
    let _outputs = MyOutputs::default();
}
