use parameter_schema_derive::EnumSchema;

#[derive(EnumSchema)]
struct NotAnEnum {
    field: i32,
}

fn main() {}
