
pub struct SortedMap<K, V> {
    keys: Vec<K>,
    vals: Vec<V>,
}

impl<K, V> SortedMap<K, V>
where
    K: Ord,
{
    pub const fn new() -> Self {
        Self {
            keys: Vec::new(),
            vals: Vec::new(),
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = self.keys.binary_search(key).ok()?;
        Some(unsafe { self.vals.get_unchecked_mut(index) })
    }

    pub fn get_or_insert_with<F>(&mut self, key: &K, f: F) -> &mut V
    where
        K: Clone,
        F: FnOnce() -> V,
    {
        match self.keys.binary_search(&key) {
            Ok(index) => unsafe { self.vals.get_unchecked_mut(index) },
            Err(index) => {
                self.keys.insert(index, key.clone());
                self.vals.insert_mut(index, f())
            }
        }
    }
}

