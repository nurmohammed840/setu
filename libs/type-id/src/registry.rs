use crate::{ComplexData, ComplexDataType, PathIdent, Type};

use std::collections::{BTreeMap, btree_map::Entry};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TypeRegistry {
    registry: BTreeMap<PathIdent, ComplexData>,
}

const INIT: ComplexData = ComplexData {
    attrs: crate::Attributes {
        docs: String::new(),
    },
    ty: ComplexDataType::Struct { fields: Vec::new() },
};

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// This method registers a type with the registry.
    ///
    /// If the type has already been registered, it will return the existing type.
    pub fn register(&mut self, name: String, init: impl FnOnce(&mut Self) -> ComplexData) -> Type {
        let path_ident = PathIdent::from(name);
        if let Entry::Vacant(entry) = self.registry.entry(path_ident.clone()) {
            entry.insert(INIT); // Mark as seen, so that recursive types can be handled.

            let user_defined_ty = init(self);
            self.registry.insert(path_ident.clone(), user_defined_ty);
        }
        Type::Complex(path_ident)
    }
}

impl std::ops::Deref for TypeRegistry {
    type Target = BTreeMap<PathIdent, ComplexData>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

impl std::ops::DerefMut for TypeRegistry {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.registry
    }
}
