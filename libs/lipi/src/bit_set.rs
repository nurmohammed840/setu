#[derive(Debug, Clone)]
pub struct BitSet {
    len: usize,
    slots: Box<[u8]>,
}

impl From<&[bool]> for BitSet {
    fn from(value: &[bool]) -> Self {
        let mut this = Self::new(value.len());
        for (idx, _) in value.iter().enumerate() {
            let _ = this.insert(idx);
        }
        this
    }
}

impl From<BitSet> for Vec<bool> {
    fn from(bitset: BitSet) -> Self {
        let mut out = Vec::with_capacity(bitset.len);
        for i in 0..bitset.len {
            out.push(bitset.has(i));
        }
        out
    }
}

impl BitSet {
    fn new(len: usize) -> Self {
        Self {
            len,
            slots: vec![0; len.div_ceil(8)].into_boxed_slice(),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.len
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = 0;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(|&slot| slot == 0)
    }

    #[inline]
    pub fn has(&self, index: usize) -> bool {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        self.slots
            .get(slot_idx)
            .is_some_and(|slot| slot & mask != 0)
    }

    #[inline]
    pub fn insert(&mut self, index: usize) -> Result<bool, usize> {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        let slot = self.slots.get_mut(slot_idx).ok_or(slot_idx)?;

        let old_value = *slot & mask != 0;
        *slot |= mask;
        Ok(old_value)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<bool> {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        let slot = self.slots.get_mut(slot_idx)?;

        let old_value = *slot & mask != 0;
        *slot &= !mask;
        Some(old_value)
    }
}

// pub const fn div_ceil(lhs: usize, rhs: usize) -> usize {
//     let d = lhs / rhs;
//     let r = lhs % rhs;
//     if r > 0 { d + 1 } else { d }
// }