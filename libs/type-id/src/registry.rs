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
    pub fn register(
        &mut self,
        name: String,
        init: impl FnOnce(&mut Self, Ident) -> ComplexData,
    ) -> Type {
        let ident = Ident(name);
        if let Entry::Vacant(entry) = self.registry.entry(ident.clone()) {
            entry.insert(ComplexData::default()); // Mark as seen, so that recursive types can be handled.

            let user_defined_ty = init(self, ident.clone());
            self.registry.insert(ident.clone(), user_defined_ty);
        }
        Type::Complex(ident)
    }
}

impl std::ops::Deref for TypeRegistry {
    type Target = BTreeMap<Ident, ComplexData>;
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
