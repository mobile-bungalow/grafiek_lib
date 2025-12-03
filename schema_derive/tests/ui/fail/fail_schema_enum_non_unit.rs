use parameter_schema_derive::EnumSchema;

#[derive(EnumSchema)]
enum HasData {
    Unit,
    Tuple(i32),
    Struct { x: i32 },
}

fn main() {}
