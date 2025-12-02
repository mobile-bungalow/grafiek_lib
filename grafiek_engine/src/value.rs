use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

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
        copy { $( $copy_variant:ident : $copy_ty:ty ),* $(,)? }
        clone { $( $clone_variant:ident : $clone_ty:ty ),* $(,)? }
    ) => {
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum Value {
            $( $copy_variant($copy_ty), )*
            $( $clone_variant($clone_ty), )*
            Null(()),
        }

        #[derive(Debug, PartialEq)]
        pub enum ValueRef<'a> {
            $( $copy_variant(&'a $copy_ty), )*
            $( $clone_variant(&'a $clone_ty), )*
            Null(()),
        }

        #[derive(Debug, PartialEq)]
        pub enum ValueMut<'a> {
            $( $copy_variant(&'a mut $copy_ty), )*
            $( $clone_variant(&'a mut $clone_ty), )*
            Null(()),
        }

        /// Checkpoint for comparing value state without cloning expensive types.
        #[derive(Debug, Clone, PartialEq)]
        pub enum ValueCheckpoint {
            $( $copy_variant($copy_ty), )*
            $( $clone_variant, )* // No data for clone types - use dirty flag
            Null,
        }

        impl Value {
            pub fn as_ref(&self) -> ValueRef<'_> {
                match self {
                    $( Value::$copy_variant(v) => ValueRef::$copy_variant(v), )*
                    $( Value::$clone_variant(v) => ValueRef::$clone_variant(v), )*
                    Value::Null(_) => ValueRef::Null(()),
                }
            }

            pub fn as_mut(&mut self) -> ValueMut<'_> {
                match self {
                    $( Value::$copy_variant(v) => ValueMut::$copy_variant(v), )*
                    $( Value::$clone_variant(v) => ValueMut::$clone_variant(v), )*
                    Value::Null(_) => ValueMut::Null(()),
                }
            }

            /// Create a checkpoint for later comparison.
            pub fn checkpoint(&self) -> ValueCheckpoint {
                match self {
                    $( Value::$copy_variant(v) => ValueCheckpoint::$copy_variant(*v), )*
                    $( Value::$clone_variant(_) => ValueCheckpoint::$clone_variant, )*
                    Value::Null(_) => ValueCheckpoint::Null,
                }
            }

            /// Check if value changed since checkpoint, clearing dirty flags.
            pub fn changed_since(&mut self, checkpoint: &ValueCheckpoint) -> bool {
                match (self, checkpoint) {
                    $( (Value::$copy_variant(v), ValueCheckpoint::$copy_variant(c)) => v != c, )*
                    $( (Value::$clone_variant(v), ValueCheckpoint::$clone_variant) => {
                        let dirty = v.is_dirty();
                        v.clear_dirty();
                        dirty
                    }, )*
                    (Value::Null(_), ValueCheckpoint::Null) => false,
                    _ => true, // Type changed
                }
            }
        }

        /// Defines the type of a given slot.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ValueType {
            $( $copy_variant, )*
            $( $clone_variant, )*
            Any,
        }

        impl Value {
            pub fn discriminant(&self) -> ValueType {
                match self {
                    $( Value::$copy_variant(_) => ValueType::$copy_variant, )*
                    $( Value::$clone_variant(_) => ValueType::$clone_variant, )*
                    Value::Null(_) => ValueType::Any,
                }
            }
        }

        impl ValueType {
            /// Returns the default value for this type
            pub fn default_value(&self) -> Value {
                match self {
                    $( ValueType::$copy_variant => Value::$copy_variant(<$copy_ty>::default()), )*
                    $( ValueType::$clone_variant => Value::$clone_variant(<$clone_ty>::default()), )*
                    ValueType::Any => Value::Null(()),
                }
            }
        }

        pub trait AsValueType {
            fn value_type() -> ValueType;
        }

        // Generate trait impls for all types
        $( define_value_enum!(@impl_traits $copy_variant, $copy_ty); )*
        $( define_value_enum!(@impl_traits $clone_variant, $clone_ty); )*
    };

    // Internal rule for trait implementations (shared by copy and clone types)
    (@impl_traits $variant:ident, $ty:ty) => {
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
    };
}

define_value_enum! {
    copy {
        I32: i32,
        F32: f32,
        Texture: TextureHandle,
    }
    clone {
        String: GrafiekString,
    }
}

/// Handle to a texture stored in the engine's texture pool.
/// The actual texture data is reference-counted by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TextureHandle(pub u32);

/// A string wrapper that requires explicit acknowledgment of changes.
/// This is because it is inefficient to compare the string on every
/// delta in immediate mode frontends.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GrafiekString {
    inner: String,
    #[serde(skip)]
    dirty: bool,
}

impl GrafiekString {
    pub fn new(s: impl Into<String>) -> Self {
        Self {
            inner: s.into(),
            dirty: false,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Get mutable access to the string. Returns a guard that must be consumed.
    pub fn edit(&mut self) -> (StringGuard<'_>, &mut String) {
        let guard = StringGuard {
            dirty: &mut self.dirty,
        };
        (guard, &mut self.inner)
    }
}

impl From<String> for GrafiekString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for GrafiekString {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Guard returned by `GrafiekString::edit()`. Must be consumed with `changed()` or `unchanged()`.
///
/// This is to prevent cloning every string multiple times and comparing per frame.
#[must_use = "StringGuard must be consumed with .changed() or .unchanged()"]
pub struct StringGuard<'a> {
    dirty: &'a mut bool,
}

impl StringGuard<'_> {
    /// Signal that the string was modified.
    pub fn changed(self) {
        *self.dirty = true;
        std::mem::forget(self);
    }

    /// Signal that the string was not modified.
    pub fn unchanged(self) {
        std::mem::forget(self);
    }
}

impl Drop for StringGuard<'_> {
    fn drop(&mut self) {
        panic!("StringGuard must be consumed with .changed() or .unchanged()");
    }
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
    /// This mirrors the cast rules in Value::cast.
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
            Value::String(s) => write!(f, "{}", s.as_str()),
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
            ValueType::String => write!(f, "string"),
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
