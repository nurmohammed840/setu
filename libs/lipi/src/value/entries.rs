use super::*;

#[derive(Clone)]
pub struct Entry {
    pub key: u16,
    pub value: Value,
}

#[derive(Clone, Default)]
pub struct Entries(Vec<Entry>);

impl From<Vec<Entry>> for Entries {
    #[inline]
    fn from(value: Vec<Entry>) -> Self {
        Self(value)
    }
}

impl Entries {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::with_capacity(8))
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    pub fn get(&self, k: u16) -> Option<&Value> {
        self.0
            .iter()
            .find_map(|Entry { key, value }| (*key == k).then_some(value))
    }

    #[inline]
    pub fn insert(&mut self, key: u16, value: Value) {
        self.0.push(Entry { key, value });
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Entry> {
        self.0.iter()
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Union({}): ", self.key)?;
        self.value.fmt(f)
    }
}

impl fmt::Display for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for Entry { key, value } in self.iter() {
            writeln!(f, "{key}: {value:#?}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Entries {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|Entry { key, value }| (key, value)))
            .finish()
    }
}
