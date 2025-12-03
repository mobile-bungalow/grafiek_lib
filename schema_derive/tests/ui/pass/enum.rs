use parameter_schema_derive::SchemaEnum;

#[derive(SchemaEnum)]
enum NotAStruct {
    A = -1,
    B = 3,
}

fn main() {}
