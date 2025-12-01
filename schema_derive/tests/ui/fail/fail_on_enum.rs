use parameter_schema_derive::InputSchema;

#[derive(InputSchema)]
enum NotAStruct {
    A,
    B,
}

fn main() {}
