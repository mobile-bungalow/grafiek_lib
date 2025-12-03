use parameter_schema_derive::EnumSchema;

#[derive(Clone, EnumSchema)]
enum CustomDefault {
    A = 1,
    B = 2,
    C = 3,
}

impl Default for CustomDefault {
    fn default() -> Self {
        Self::B
    }
}

fn main() {}
