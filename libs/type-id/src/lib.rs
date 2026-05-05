mod registry;
mod types;
mod utils;

pub use registry::TypeRegistry;
pub use type_id_macros::TypeId;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub trait TypeId {
    fn ty(_: &mut TypeRegistry) -> Type;
}

#[derive(Debug, Clone, Hash)]
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
    Complex(Ident),
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ListVariant {
    BTreeSet,
    HashSet,
    BinaryHeap,
    LinkedList,
    VecDeque,
    Vec,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MapVariant {
    HashMap,
    BTreeMap,
}

// ===========================================================

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ident(pub String);

macro_rules! discriminant {
    [$($id:tt : $ty:ty)*] => {
        #[derive(Debug, Clone, Hash)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        pub enum Discriminant { $($id($ty)),* }
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

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Attribute {
    Docs(String),
}

#[derive(Default, Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Attributes {
    pub data: Vec<Attribute>,
}

// ===========================================================

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ComplexData {
    pub attrs: Attributes,
    pub name: Ident,
    // pub generics: Generics,
    pub ty: ComplexDataType,
}

impl Default for ComplexData {
    fn default() -> Self {
        Self {
            attrs: Attributes::default(),
            name: Ident(String::new()),
            ty: ComplexDataType::Struct { fields: vec![] },
        }
    }
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ComplexDataType {
    Enum {
        fields: Vec<(Attributes, EnumField)>,
    },
    Struct {
        fields: Vec<(Attributes, StructField)>,
    },
    Tuple {
        fields: Vec<(Attributes, Type)>,
    },
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumField {
    pub name: Ident,
    pub ty: EnumFieldType,
    pub discriminant: Option<Discriminant>,
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EnumFieldType {
    Unit,
    Struct(Vec<StructField>),
    Tuple(Vec<Type>),
}

#[derive(Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StructField {
    pub name: Ident,
    pub ty: Type,
}
