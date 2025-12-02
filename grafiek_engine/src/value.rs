use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::fmt;
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

// TODO: This is a psycho loadbearing macro I would reallllly
// like to prune this down and the traits involved if possible
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

        #[derive(Debug, Clone, Copy, PartialEq)]
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

            /// Create a checkpoint for later comparison.
            ///
            /// TODO: we want to opt this into a different type later
            /// As we may just store a length hash pair for heapy types
            pub fn checkpoint(&self) -> Value {
                self.clone()
            }

            /// Check if value changed since checkpoint.
            pub fn changed_since(&self, checkpoint: &Value) -> bool {
                self != checkpoint
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

        impl ValueType {
            /// Returns the default value for this type
            pub fn default_value(&self) -> Value {
                match self {
                    $(
                        ValueType::$variant => Value::$variant(<$ty>::default()),
                    )*
                    ValueType::Any => Value::Null(()),
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
                const VALUE_TYPE: ValueType = ValueType::$variant;
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

            impl Extract for $ty {
                fn extract(value: ValueRef<'_>) -> Result<Self, ValueError> {
                    match value {
                        ValueRef::$variant(v) => Ok(*v),
                        other => Err(ValueError::TypeMismatch {
                            wanted: stringify!($variant).to_string(),
                            found: format!("{:?}", other),
                        }),
                    }
                }
            }

            impl ExtractMut for $ty {
                fn extract_mut<'a>(value: &'a mut ValueMut<'_>) -> Result<&'a mut Self, ValueError> {
                    match value {
                        ValueMut::$variant(v) => Ok(v),
                        other => Err(ValueError::TypeMismatch {
                            wanted: stringify!($variant).to_string(),
                            found: format!("{:?}", other),
                        }),
                    }
                }
            }
        )*

    };
}

pub trait AsValueType {
    const VALUE_TYPE: ValueType;
    fn value_type() -> ValueType {
        Self::VALUE_TYPE
    }
}

pub trait Extract: Sized + Copy {
    fn extract(value: ValueRef<'_>) -> Result<Self, ValueError>;
}

pub trait ExtractMut: Sized {
    fn extract_mut<'a>(value: &'a mut ValueMut<'_>) -> Result<&'a mut Self, ValueError>;
}

/// Handle to a texture stored in the engine's texture pool.
/// The actual texture data is reference-counted by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
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

    /// Check if a value of this type can be cast to the target type.
    /// This is the single source of truth for cast compatibility rules.
    pub fn can_cast_to(&self, target: &ValueType) -> bool {
        match (self, target) {
            (_, ValueType::Any) => true,
            (ValueType::Any, _) => true,
            (a, b) if a == b => true,
            (ValueType::I32, ValueType::F32) => true,
            (ValueType::F32, ValueType::I32) => true,
            _ => false,
        }
    }
}

impl Value {
    /// Cast this value to the target type.
    /// Returns None if the cast is not supported.
    pub fn cast(&self, target: &ValueType) -> Option<Value> {
        // Null can never be cast to anything
        if matches!(self, Value::Null(_)) {
            return None;
        }

        // Check type compatibility first
        if !self.discriminant().can_cast_to(target) {
            return None;
        }

        // Perform the actual conversion
        Some(match (self, target) {
            (_, ValueType::Any) => self.clone(),
            (Value::I32(i), ValueType::F32) => Value::F32(*i as f32),
            (Value::F32(f), ValueType::I32) => Value::I32(f.trunc() as i32),
            // Identity cast - type already matches
            _ => self.clone(),
        })
    }

    pub fn can_cast_to(&self, ty: &ValueType) -> bool {
        // Null is a special case that can_cast_to on ValueType doesn't handle
        if matches!(self, Value::Null(_)) {
            return false;
        }
        self.discriminant().can_cast_to(ty)
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

pub type Inputs<'a> = ArrayVec<ValueRef<'a>, MAX_SLOTS>;
pub type Outputs<'a> = ArrayVec<ValueMut<'a>, MAX_SLOTS>;

// TODO: Alotta traits dude
pub trait InputsExt {
    fn extract<T: Extract>(&self, index: usize) -> Result<T, ValueError>;
}

impl InputsExt for Inputs<'_> {
    fn extract<T: Extract>(&self, index: usize) -> Result<T, ValueError> {
        let value = self.get(index).ok_or(ValueError::Index(index))?;
        T::extract(*value)
    }
}

// TODO: Alotta traits dude
pub trait OutputsExt {
    fn extract<T: ExtractMut>(&mut self, index: usize) -> Result<&mut T, ValueError>;
}

impl OutputsExt for Outputs<'_> {
    fn extract<T: ExtractMut>(&mut self, index: usize) -> Result<&mut T, ValueError> {
        let slot = self.get_mut(index).ok_or(ValueError::Index(index))?;
        T::extract_mut(slot)
    }
}
