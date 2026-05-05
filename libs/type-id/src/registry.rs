use crate::{ComplexData, Ident, Type};

use std::collections::{BTreeMap, btree_map::Entry};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeRegistry {
    registry: BTreeMap<Ident, ComplexData>,
}

impl TypeRegistry {
    /// This method registers a type with the registry.
    /// 
    /// If the type has already been registered, it will return the existing type.
    pub fn register(&mut self, name: Ident, init: fn(&mut Self) -> ComplexData) -> Type {
        if let Entry::Vacant(entry) = self.registry.entry(name.clone()) {
            entry.insert(ComplexData::default()); // Mark as seen, so that recursive types can be handled.
            let user_defined_ty = init(self);
            self.registry.insert(name.clone(), user_defined_ty);
        }
        Type::Complex(name)
    }
}
