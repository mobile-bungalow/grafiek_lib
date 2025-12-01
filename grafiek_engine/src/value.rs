use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::fmt;
use std::ops::{Deref, DerefMut};
use thiserror::Error;

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

/// A typesafe guard over a value that tracks mutations.
/// On drop, sets a shared dirty flag if the value was modified and writes back to the Value.
pub struct ValueGuard<'a, T>
where
    T: Clone + PartialEq,
    Value: From<T>,
{
    value: T,
    original: T,
    slot: &'a mut Value,
    dirty: &'a Cell<bool>,
    pub metadata: &'a SlotMetadata,
}

impl<'a, T> ValueGuard<'a, T>
where
    T: Clone + PartialEq,
    for<'b> &'b mut T: TryFrom<&'b mut Value, Error = ValueError>,
    Value: From<T>,
{
    pub(crate) fn new(
        slot: &'a mut Value,
        dirty: &'a Cell<bool>,
        metadata: &'a SlotMetadata,
    ) -> Result<Self, ValueError> {
        let value: T = {
            let ref_mut: &mut T = (&mut *slot).try_into()?;
            ref_mut.clone()
        };
        let original = value.clone();
        Ok(Self {
            value,
            original,
            slot,
            dirty,
            metadata,
        })
    }
}

impl<T> Deref for ValueGuard<'_, T>
where
    T: Clone + PartialEq,
    Value: From<T>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for ValueGuard<'_, T>
where
    T: Clone + PartialEq,
    Value: From<T>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Drop for ValueGuard<'_, T>
where
    T: Clone + PartialEq,
    Value: From<T>,
{
    fn drop(&mut self) {
        if self.value != self.original {
            self.dirty.set(true);
            *self.slot = Value::from(self.value.clone());
        }
    }
}

/// Read-only view into input values for [Operation::execute]
pub struct Inputs<'a>(&'a [Value]);

impl<'a> Inputs<'a> {
    pub fn new(values: &'a [Value]) -> Self {
        Self(values)
    }

    /// Get a typed reference to an input value
    pub fn get<T>(&self, index: usize) -> Result<&T, ValueError>
    where
        for<'b> &'b T: TryFrom<&'b Value, Error = ValueError>,
    {
        self.0
            .get(index)
            .ok_or(ValueError::Index(index))?
            .try_into()
    }

    /// Number of input slots
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the raw Value at an index
    pub fn raw(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Iterate over raw values
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.0.iter()
    }
}

/// Mutable view into output values for execute
pub struct Outputs<'a>(&'a mut [Value]);

impl<'a> Outputs<'a> {
    pub fn new(values: &'a mut [Value]) -> Self {
        Self(values)
    }

    /// Set an output value, checking that the type matches the slot's declared type
    pub fn set<T>(&mut self, index: usize, val: T) -> Result<(), ValueError>
    where
        Value: From<T>,
    {
        let slot = self.0.get_mut(index).ok_or(ValueError::Index(index))?;

        let new_val = Value::from(val);
        if std::mem::discriminant(slot) != std::mem::discriminant(&new_val) {
            return Err(ValueError::TypeMismatch {
                wanted: format!("{:?}", slot.discriminant()),
                found: format!("{:?}", new_val.discriminant()),
            });
        }
        *slot = new_val;
        Ok(())
    }

    /// Number of output slots
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the raw Value at an index
    pub fn raw(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    /// Get a mutable reference to the raw Value,
    /// You really should try NOT to change the type of a value!
    /// This is not anticipated for during graph calculation.
    pub unsafe fn raw_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.0.get_mut(index)
    }

    /// Iterate over raw values
    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.0.iter()
    }
}
