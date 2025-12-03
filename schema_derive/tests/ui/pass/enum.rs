use parameter_schema_derive::EnumSchema;

#[derive(Default, Clone, EnumSchema)]
enum NotAStruct {
    #[default]
    A = -1,
    B = 3,
}

fn main() {}
