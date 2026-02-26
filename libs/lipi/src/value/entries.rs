use super::*;

#[derive(Clone)]
pub struct Entry {
    pub key: u16,
    pub value: Value,
}

#[derive(Clone, Default)]
pub struct Struct(pub Array<Entry>);

impl<T: Into<Array<Entry>>> From<T> for Struct {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Struct {
    pub fn get(&self, k: u16) -> Option<&Value> {
        self.0
            .iter()
            .find_map(|Entry { key, value }| (*key == k).then_some(value))
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

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for Entry { key, value } in self.iter() {
            writeln!(f, "{key}: {value:#?}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.iter().map(|Entry { key, value }| (key, value)))
            .finish()
    }
}
