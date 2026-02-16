use crate::{Value, convert::ConvertFrom, errors};

#[derive(Clone)]
pub struct Entry<'de> {
    pub key: u16,
    pub value: Value<'de>,
}

impl<'de> std::fmt::Debug for Entry<'de> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Union({}): ", self.key)?;
        self.value.fmt(f)
    }
}

#[derive(Clone, Default)]
pub struct Entries<'de>(Vec<Entry<'de>>);

impl<'de> From<Vec<Entry<'de>>> for Entries<'de> {
    #[inline]
    fn from(value: Vec<Entry<'de>>) -> Self {
        Self(value)
    }
}

impl<'de> Entries<'de> {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::with_capacity(8))
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn get(&self, k: u16) -> Option<&Value<'de>> {
        self.0
            .iter()
            .find_map(|Entry { key, value }| (*key == k).then_some(value))
    }

    #[inline]
    pub fn insert(&mut self, key: u16, value: Value<'de>) {
        self.0.push(Entry { key, value });
    }

    pub fn get_and_convert<'v, T>(&'v self, k: u16) -> Result<T, errors::ConvertError>
    where
        T: ConvertFrom<Option<&'v Value<'de>>>,
    {
        T::convert_from(self.get(k)).map_err(|mut err| {
            err.key = Some(k);
            err
        })
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Entry<'de>> {
        self.0.iter()
    }
}
