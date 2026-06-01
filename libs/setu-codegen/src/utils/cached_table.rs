use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
    rc::Rc,
};

#[derive(Debug, Default)]
pub struct LocalCachedTable<K, V: ?Sized> {
    map: RefCell<HashMap<K, Rc<V>>>,
}

impl<K: Eq + Hash, V: ?Sized> LocalCachedTable<K, V> {
    pub fn new() -> Self {
        Self {
            map: RefCell::default(),
        }
    }

    pub fn get_or_insert_with<T>(&self, k: K, f: impl FnOnce() -> T) -> Rc<V>
    where
        Rc<V>: From<T>,
    {
        match self.map.borrow_mut().entry(k) {
            Entry::Occupied(cached) => cached.get().clone(),
            Entry::Vacant(map) => {
                let val: Rc<V> = Rc::from(f());
                map.insert(val.clone());
                val
            }
        }
    }

    pub fn _get_by_ref_or_insert_with<'k, Q, T>(&self, k: &'k Q, f: impl FnOnce() -> T) -> Rc<V>
    where
        K: Borrow<Q>,
        K: From<&'k Q>,
        Q: Hash + Eq + ?Sized,
        Rc<V>: From<T>,
    {
        let mut map = self.map.borrow_mut();
        if let Some(cached) = map.get(k) {
            return cached.clone();
        }
        let val: Rc<V> = Rc::from(f());
        map.insert(k.into(), val.clone());
        val
    }
}
