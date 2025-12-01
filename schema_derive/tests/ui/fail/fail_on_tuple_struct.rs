use parameter_schema_derive::InputSchema;

#[derive(InputSchema)]
struct TupleStruct(f32, i32);

fn main() {}
