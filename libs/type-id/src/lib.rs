mod registry;
mod types;
mod utils;

use std::{any::Any, fmt, sync::Arc};

pub use registry::TypeRegistry;
pub use type_id_macros::TypeId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait TypeId {
    fn ty(_: &mut TypeRegistry) -> Type;
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Type {
    // ===== Numbers =====
    U8,
    U16,
    U32,
    U64,
    U128,

    I8,
    I16,
    I32,
    I64,
    I128,

    F32,
    F64,

    // ===== Common =====
    Bool,
    Char,
    String,

    // ===== STD =====
    Option(Box<Type>),
    Result(Box<(Type, Type)>),
    Other(OtherType),

    // ===== Compound =====
    Array {
        ty: Box<Type>,
        len: usize,
    },
    List {
        variant: ListVariant,
        ty: Box<Type>,
    },
    Map {
        variant: MapVariant,
        ty: Box<(Type, Type)>,
    },
    Tuple(Vec<Type>),

    /// The path of the user defined type
    ///
    /// ```ignore
    ///               struct Bar { ... }     enum Foo { ... }
    ///                      ^^^                  ^^^
    ///                         \                /
    /// Type::Complex("<path>::Bar" | "<path>::Foo")
    /// ```
    Complex(PathIdent),
}

#[derive(Debug, Clone)]
pub struct OtherType(pub Arc<dyn Any + Send + Sync>);

impl PartialEq for OtherType {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

macro_rules! option {
    [$self:expr, $pattern:pat => $val:expr] => {
        match $self {
            $pattern => Some($val),
            _ => None,
        }
    };
}

impl Type {
    pub fn optional(&self) -> Option<&Self> {
        option!(self, Self::Option(ty) => ty)
    }

    pub fn complex(&self) -> Option<&PathIdent> {
        option!(self, Self::Complex(ty) => ty)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ListVariant {
    BTreeSet,
    HashSet,
    BinaryHeap,
    LinkedList,
    VecDeque,
    Vec,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MapVariant {
    HashMap,
    BTreeMap,
}

// ===========================================================

/// Cheap Clone
pub type PathIdent = Arc<str>;
pub type Ident = Box<str>;

macro_rules! discriminant {
    [$($id:tt : $ty:ty)*] => {
        #[derive(Debug, Clone)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub enum Discriminant {
            $($id($ty),)*
            None
        }
        $(
            impl From<$ty> for Discriminant {
                fn from(v: $ty) -> Self {
                    Self::$id(v)
                }
            }
        )*
    };
}

discriminant! {
    U8: u8
    U16: u16
    U32: u32
    U64: u64
    I8: i8
    I16: i16
    I32: i32
    I64: i64
}

impl fmt::Display for Discriminant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Discriminant::U8(v) => v.fmt(f),
            Discriminant::U16(v) => v.fmt(f),
            Discriminant::U32(v) => v.fmt(f),
            Discriminant::U64(v) => v.fmt(f),
            Discriminant::I8(v) => v.fmt(f),
            Discriminant::I16(v) => v.fmt(f),
            Discriminant::I32(v) => v.fmt(f),
            Discriminant::I64(v) => v.fmt(f),
            Discriminant::None => Ok(()),
        }
    }
}

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Attributes {
    pub docs: String,
}

impl Attributes {
    pub fn docs<T: Into<String>>(docs: T) -> Attributes {
        Attributes { docs: docs.into() }
    }
}

// ===========================================================

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ComplexData {
    pub attrs: Attributes,
    pub ty: ComplexDataType,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ComplexDataType {
    Enum {
        is_numeric: bool,
        fields: Vec<(Attributes, EnumField)>,
    },
    Struct {
        fields: Vec<(Attributes, StructField)>,
    },
    Tuple {
        fields: Vec<(Attributes, Type)>,
    },
}

impl ComplexDataType {
    pub fn as_struct(fields: Vec<(Attributes, StructField)>) -> Self {
        Self::Struct { fields }
    }

    pub fn as_tuple(fields: Vec<(Attributes, Type)>) -> Self {
        Self::Tuple { fields }
    }

    pub fn as_enum(is_numeric: bool, fields: Vec<(Attributes, EnumField)>) -> Self {
        Self::Enum { is_numeric, fields }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumField {
    pub name: Ident,
    pub ty: EnumFieldType,
    pub discriminant: Discriminant,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EnumFieldType {
    Unit,
    Struct(Vec<(Attributes, StructField)>),
    Tuple(Vec<(Attributes, Type)>),
}

impl EnumFieldType {
    pub fn as_struct(fields: Vec<(Attributes, StructField)>) -> Self {
        Self::Struct(fields)
    }

    pub fn as_tuple(fields: Vec<(Attributes, Type)>) -> Self {
        Self::Tuple(fields)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StructField {
    pub key: u32,
    pub name: Ident,
    pub ty: Type,
}
