use super::sorted_map::SortedMap;
use std::any::Any;

pub struct Store {
    map: SortedMap<u16, Box<dyn Any>>,
}

impl Store {
    pub const fn new() -> Self {
        Self {
            map: SortedMap::new(),
        }
    }

    pub fn get_mut<T: 'static>(&mut self, key: u16) -> Option<&mut T> {
        self.map.get_mut(&key)?.downcast_mut()
    }

    pub fn get_or_insert_with<F, T: 'static>(&mut self, key: u16, f: F) -> &mut T
    where
        F: FnOnce() -> T,
        F::Output: Into<Box<T>>,
    {
        self.map
            .get_or_insert_with(&key, || Box::from(f()))
            .downcast_mut()
            .unwrap()
    }
}
