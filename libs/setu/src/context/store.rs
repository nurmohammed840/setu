use std::any::{Any, TypeId};

pub struct Store {
    vals: Vec<Box<dyn Any>>,
}

impl std::fmt::Debug for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store").finish()
    }
}

impl Store {
    pub const fn new() -> Self {
        Self { vals: Vec::new() }
    }

    fn binary_search<T: 'static>(&self) -> Result<usize, usize> {
        let id = TypeId::of::<T>();
        self.vals
            .binary_search_by_key(&id, |v| v.as_ref().type_id())
    }

    pub fn get<T: 'static>(&mut self) -> Option<&mut T> {
        let index = self.binary_search::<T>().ok()?;
        let any = unsafe { self.vals.get_unchecked_mut(index) };
        Some(unsafe { any.downcast_mut().unwrap_unchecked() })
    }

    pub fn init<F, T: 'static>(&mut self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
        F::Output: Into<Box<T>>,
    {
        let any = match self.binary_search::<T>() {
            Ok(index) => unsafe { self.vals.get_unchecked_mut(index) },
            Err(index) => self.vals.insert_mut(index, Box::from(f())),
        };
        unsafe { any.downcast_mut().unwrap_unchecked() }
    }

    pub fn take<T: 'static>(&mut self) -> Option<Box<T>> {
        let index = self.binary_search::<T>().ok()?;
        let any = self.vals.remove(index);
        Some(unsafe { any.downcast().unwrap_unchecked() })
    }
}
