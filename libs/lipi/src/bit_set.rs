#[derive(Debug, Clone)]
pub struct BitSet<Bytes> {
    len: usize,
    slots: Bytes,
}

impl<Bytes: AsRef<[u8]>> BitSet<Bytes> {
    #[inline]
    pub fn capacity(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slots.as_ref().iter().all(|&slot| slot == 0)
    }

    #[inline]
    pub fn has(&self, index: usize) -> bool {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        self.slots
            .as_ref()
            .get(slot_idx)
            .is_some_and(|slot| slot & mask != 0)
    }

    pub fn as_packed_bytes(&self) -> &[u8] {
        self.slots.as_ref()
    }
}

impl<Bools, Bytes> From<Bools> for BitSet<Bytes>
where
    Bools: AsRef<[bool]>,
    Bytes: From<Vec<u8>>,
{
    fn from(value: Bools) -> Self {
        let bools = value.as_ref();
        let mut this = BitSet::<Vec<_>>::new(bools.len());

        for (idx, b) in bools.iter().enumerate() {
            if *b {
                let _ = this.insert(idx);
            }
        }
        BitSet {
            len: this.len,
            slots: Bytes::from(this.slots),
        }
    }
}

impl<Bytes: AsMut<[u8]>> BitSet<Bytes> {
    pub fn clear(&mut self) {
        for slot in self.slots.as_mut() {
            *slot = 0;
        }
    }

    #[inline]
    pub fn insert(&mut self, index: usize) -> Result<bool, usize> {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        let slot = self.slots.as_mut().get_mut(slot_idx).ok_or(slot_idx)?;

        let old_value = *slot & mask != 0;
        *slot |= mask;
        Ok(old_value)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Option<bool> {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        let slot = self.slots.as_mut().get_mut(slot_idx)?;

        let old_value = *slot & mask != 0;
        *slot &= !mask;
        Some(old_value)
    }
}

impl BitSet<Box<[u8]>> {
    pub fn new(len: usize) -> Self {
        Self {
            len,
            slots: vec![0; len.div_ceil(8)].into_boxed_slice(),
        }
    }

    pub fn from_packed(bytes: Box<[u8]>) -> Self {
        Self {
            len: bytes.len() * 8,
            slots: bytes,
        }
    }
}

impl BitSet<Vec<u8>> {
    pub fn new(len: usize) -> Self {
        Self {
            len,
            slots: vec![0; len.div_ceil(8)],
        }
    }

    pub fn from_packed(bytes: Vec<u8>) -> Self {
        Self {
            len: bytes.len() * 8,
            slots: bytes,
        }
    }
}

impl<Bytes: AsRef<[u8]>> From<BitSet<Bytes>> for Vec<bool> {
    fn from(bitset: BitSet<Bytes>) -> Self {
        let mut out = Vec::with_capacity(bitset.len);
        for i in 0..bitset.len {
            out.push(bitset.has(i));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::bit_set::BitSet;
    use std::borrow::Cow;

    #[test]
    fn create_bit_set() {
        let bit_set_1 = BitSet::<Box<[u8]>>::from(&[true, false]);
        let bit_set_2 = BitSet::<Cow<[u8]>>::from(&[true, false]);
        assert_eq!(bit_set_1.as_packed_bytes(), bit_set_2.as_packed_bytes());

        let mut bit_set_3 = BitSet::<Vec<u8>>::new(2);
        bit_set_3.insert(0).unwrap();
        assert_eq!(bit_set_1.as_packed_bytes(), bit_set_3.as_packed_bytes());
    }

    #[test]
    fn packed_bool_len() {
        assert_eq!(0_u8.div_ceil(8), 0);
        assert_eq!(1_u8.div_ceil(8), 1);
        assert_eq!(8_u8.div_ceil(8), 1);
        assert_eq!(9_u8.div_ceil(8), 2);
        assert_eq!(16u8.div_ceil(8), 2);

        for i in 0..=256 as usize {
            assert_eq!(i.div_ceil(8), (i + 7) / 8);
        }
    }

    #[test]
    fn has_and_is_lsb_first() {
        let mut bs = BitSet::<Vec<u8>>::new(8);

        bs.insert(0).unwrap();
        assert_eq!(bs.as_packed_bytes(), &[0b0000_0001]);

        bs.insert(1).unwrap();
        assert_eq!(bs.as_packed_bytes(), &[0b0000_0011]);

        bs.insert(7).unwrap();
        assert_eq!(bs.as_packed_bytes(), &[0b1000_0011]);

        assert!(bs.has(0));
        assert!(bs.has(1));
        assert!(bs.has(7));
        assert!(!bs.has(2));
    }

    #[test]
    fn insert_and_returns_old_value() {
        let mut bs = BitSet::<Vec<u8>>::new(16);

        assert_eq!(bs.insert(3).unwrap(), false);
        assert!(bs.has(3));

        assert_eq!(bs.insert(3).unwrap(), true);
        assert!(bs.has(3));
    }

    #[test]
    fn remove_and_returns_old_value() {
        let mut bs = BitSet::<Vec<u8>>::new(16);

        assert_eq!(bs.remove(5), Some(false));
        assert!(!bs.has(5));

        assert_eq!(bs.insert(5).unwrap(), false);
        assert!(bs.has(5));

        assert_eq!(bs.remove(5), Some(true));
        assert!(!bs.has(5));
    }

    #[test]
    fn out_of_bounds_insert_and_remove() {
        let mut bs = BitSet::<Vec<u8>>::new(8);

        // slots.len() == 1, so index 8 => slot_idx 1 (OOB)
        let err = bs.insert(8).unwrap_err();
        assert_eq!(err, 1);

        assert_eq!(bs.remove(8), None);

        assert_eq!(bs.has(8), false);
        assert_eq!(bs.has(9999), false);
    }

    #[test]
    fn bools_roundtrip() {
        let input = vec![
            true, false, true, true, false, false, false, true, // first byte
            false, true, false, false, true, false, true, false, // second byte
            true,  // third byte partially used
        ];

        let bs = BitSet::<Vec<u8>>::from(&input);
        let output: Vec<bool> = Vec::from(bs);

        assert_eq!(output, input);
    }

    #[test]
    fn cross_byte_boundaries() {
        // boundary at 8 bits
        let mut bs = BitSet::<Vec<u8>>::new(16);

        bs.insert(7).unwrap();
        bs.insert(8).unwrap();
        bs.insert(15).unwrap();

        assert!(bs.has(7));
        assert!(bs.has(8));
        assert!(bs.has(15));

        // byte0: bit7 set => 0b1000_0000
        // byte1: bit0 + bit7 set => 0b1000_0001
        assert_eq!(bs.as_packed_bytes(), &[0b1000_0000, 0b1000_0001]);
    }
}
