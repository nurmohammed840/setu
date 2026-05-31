use std::hash::Hash;
use std::rc::Rc;
use std::{borrow::Borrow, cell::RefCell, collections::HashMap};

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

    pub fn get_or_insert_with<'k, Q, T>(&self, k: &'k Q, f: impl FnOnce() -> T) -> Rc<V>
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
