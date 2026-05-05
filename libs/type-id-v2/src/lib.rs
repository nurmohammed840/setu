mod registry;
mod types;
mod utils;

pub use registry::TypeRegistry;

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
    // ===== Complex =====
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
