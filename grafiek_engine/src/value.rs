use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

/// Maximum number of input/output slots per node
pub const MAX_SLOTS: usize = 32;

#[derive(Error, Debug)]
pub enum ValueError {
    #[error("Slot index {0} does not exist")]
    Index(usize),

    #[error("Type mismatch: wanted {wanted}, found {found}")]
    TypeMismatch { wanted: String, found: String },
}

macro_rules! define_value_enum {
    (
        $(
            $variant:ident : $ty:ty
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum Value {
            $(
                $variant($ty),
            )*
            Null(()),
        }

        #[derive(Debug, PartialEq)]
        pub enum ValueRef<'a> {
            $(
                $variant(&'a $ty),
            )*
            Null(()),
        }

        #[derive(Debug, PartialEq)]
        pub enum ValueMut<'a> {
            $(
                $variant(&'a mut $ty),
            )*
            Null(()),
        }

        impl Value {
            pub fn as_ref(&self) -> ValueRef<'_> {
                match self {
                    $(
                        Value::$variant(v) => ValueRef::$variant(v),
                    )*
                    Value::Null(_) => ValueRef::Null(()),
                }
            }

            pub fn as_mut(&mut self) -> ValueMut<'_> {
                match self {
                    $(
                        Value::$variant(v) => ValueMut::$variant(v),
                    )*
                    Value::Null(_) => ValueMut::Null(()),
                }
            }
        }

        /// Defines the type of a given slot.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ValueType {
            $(
                $variant,
            )*
            Any,
        }

        impl Value {
            pub fn discriminant(&self) -> ValueType {
                match self {
                    $(
                        Value::$variant(_) => ValueType::$variant,
                    )*
                    Value::Null(_) => ValueType::Any,
                }
            }
        }

        $(
            impl From<$ty> for Value {
                fn from(v: $ty) -> Self {
                    Value::$variant(v)
                }
            }

            impl AsValueType for $ty {
                fn value_type() -> ValueType {
                    ValueType::$variant
                }
            }

            impl<'a> TryFrom<&'a mut Value> for &'a mut $ty {
                type Error = ValueError;
                fn try_from(v: &'a mut Value) -> Result<Self, Self::Error> {
                    let Value::$variant(v) = v else {
                        return Err(ValueError::TypeMismatch {
                            wanted: format!("{:?}", <$ty>::value_type()),
                            found: format!("{:?}", v),
                        });
                    };

                    Ok(v)
                }
            }

            impl<'a> TryFrom<&'a Value> for &'a $ty {
                type Error = ValueError;
                fn try_from(v: &'a Value) -> Result<Self, Self::Error> {
                    let Value::$variant(v) = v else {
                        return Err(ValueError::TypeMismatch {
                            wanted: format!("{:?}", <$ty>::value_type()),
                            found: format!("{:?}", v),
                        });
                    };

                    Ok(v)
                }
            }
        )*

    };
}

pub trait AsValueType {
    fn value_type() -> ValueType;
}

/// Handle to a texture stored in the engine's texture pool.
/// The actual texture data is reference-counted by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureHandle(pub u32);

define_value_enum! {
    I32: i32,
    F32: f32,
    Texture: TextureHandle,
}

impl ValueType {
    /// Check if this type matches another type, considering Any as a wildcard
    pub fn matches(&self, other: &ValueType) -> bool {
        match (self, other) {
            (ValueType::Any, _) | (_, ValueType::Any) => true,
            _ => self == other,
        }
    }
}

impl Value {
    /// Cast this value to the target type
    /// Returns None if the cast is not supported
    pub fn cast(&self, ty: &ValueType) -> Option<Value> {
        match (self, ty) {
            // Null can never be passed as an argument to anything, even Any types
            // So it's appropriate to fail cast here
            (Value::Null(_), _) => None,
            (_, ValueType::Any) => Some(self.clone()),
            // Identity casts
            (Value::I32(_), ValueType::I32)
            | (Value::F32(_), ValueType::F32)
            | (Value::Texture(_), ValueType::Texture) => Some(self.clone()),
            // Numeric conversions
            (Value::I32(i), ValueType::F32) => Some(Value::F32(*i as f32)),
            (Value::F32(f), ValueType::I32) => Some(Value::I32(f.trunc() as i32)),
            // Texture cannot be cast to/from numeric types
            _ => None,
        }
    }

    pub fn can_cast_to(&self, ty: &ValueType) -> bool {
        self.cast(ty).is_some()
    }
}

// TODO: we should probably make this derived or define it inside the macro
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::I32(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{:.3}", v),
            Value::Texture(h) => write!(f, "texture({})", h.0),
            Value::Null(_) => write!(f, "null"),
        }
    }
}

// TODO: we should probably make this derived or define it inside the macro
impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueType::I32 => write!(f, "i32"),
            ValueType::F32 => write!(f, "f32"),
            ValueType::Texture => write!(f, "texture"),
            ValueType::Any => write!(f, "any"),
        }
    }
}

/// Read-only view into input values for [Operation::execute]
pub type Inputs<'a> = ArrayVec<ValueRef<'a>, MAX_SLOTS>;

/// Mutable view into output values for [Operation::execute]
pub type Outputs<'a> = ArrayVec<ValueMut<'a>, MAX_SLOTS>;

/// Collect immutable views from a slice of Values
pub fn inputs_from_slice(values: &[Value]) -> Inputs<'_> {
    values.iter().map(Value::as_ref).collect()
}

/// Collect mutable views from a slice of Values
pub fn outputs_from_slice(values: &mut [Value]) -> Outputs<'_> {
    values.iter_mut().map(Value::as_mut).collect()
}
