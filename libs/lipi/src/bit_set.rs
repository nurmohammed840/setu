use crate::utils;

pub trait BitSet {
    fn capacity(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn has(&self, index: usize) -> bool;
    fn get(&self, index: usize) -> Option<bool>;
}

pub trait BitSetMut {
    fn clear(&mut self);
    fn set(&mut self, index: usize) -> Result<bool, usize>;
    fn remove(&mut self, index: usize) -> Option<bool>;
}

impl<Bytes> BitSet for Bytes
where
    Bytes: AsRef<[u8]> + ?Sized,
{
    fn capacity(&self) -> usize {
        self.as_ref().len() * 8
    }

    fn is_empty(&self) -> bool {
        self.as_ref().iter().all(|&slot| slot == 0)
    }

    #[inline]
    fn has(&self, index: usize) -> bool {
        self.get(index).unwrap_or_default()
    }

    #[inline]
    fn get(&self, index: usize) -> Option<bool> {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        let slot = self.as_ref().get(slot_idx)?;
        Some(slot & mask != 0)
    }
}

impl<Bytes> BitSetMut for Bytes
where
    Bytes: AsMut<[u8]> + ?Sized,
{
    fn clear(&mut self) {
        for slot in self.as_mut() {
            *slot = 0;
        }
    }

    #[inline]
    fn set(&mut self, index: usize) -> Result<bool, usize> {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        let slot = self.as_mut().get_mut(slot_idx).ok_or(slot_idx)?;

        let old_value = *slot & mask != 0;
        *slot |= mask;
        Ok(old_value)
    }

    #[inline]
    fn remove(&mut self, index: usize) -> Option<bool> {
        let slot_idx = index / u8::BITS as usize;
        let mask = 1 << (index % u8::BITS as usize);
        let slot = self.as_mut().get_mut(slot_idx)?;

        let old_value = *slot & mask != 0;
        *slot &= !mask;
        Some(old_value)
    }
}

pub fn bitvec(len: usize) -> Vec<u8> {
    vec![0; utils::bool_packed_len(len)]
}

pub fn bitvec_from(bools: &[bool]) -> Vec<u8> {
    let mut bv = bitvec(bools.len());
    for (idx, &bool) in bools.iter().enumerate() {
        if bool {
            let _ = bv.set(idx);
        }
    }
    bv
}

pub fn bitvec_to_bools(len: usize, bitvec: &[u8]) -> Vec<bool> {
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        out.push(BitSet::has(bitvec, i));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_bit_set() {
        let bs_1 = bitvec_from(&[true, false]);
        let bs_2 = bitvec_from(&[true, false]);
        assert_eq!(bs_1, bs_2);

        let mut bs_3 = bitvec(2);
        bs_3.set(0).unwrap();
        assert_eq!(bs_1, bs_3);
    }

    #[test]
    fn get_behavior() {
        let mut bs = bitvec(8);

        assert_eq!(bs.get(8), None);

        bs.set(0).unwrap();
        bs.set(7).unwrap();
        assert_eq!(bs.get(0), Some(true));
        assert_eq!(bs.get(7), Some(true));

        assert_eq!(bs.get(999), None);

        assert!(!bs.is_empty());
        bs.clear();
        assert!(bs.is_empty());
    }

    #[test]
    fn has_and_is_lsb_first() {
        let mut bs = bitvec(8);

        bs.set(0).unwrap();
        assert_eq!(bs, &[0b0000_0001]);

        bs.set(1).unwrap();
        assert_eq!(bs, &[0b0000_0011]);

        bs.set(7).unwrap();
        assert_eq!(bs, &[0b1000_0011]);

        assert!(bs.has(0));
        assert!(bs.has(1));
        assert!(bs.has(7));
        assert!(!bs.has(2));
    }

    #[test]
    fn insert_and_returns_old_value() {
        let mut bs = bitvec(16);

        assert_eq!(bs.set(3), Ok(false));
        assert!(bs.has(3));

        assert_eq!(bs.set(3), Ok(true));
        assert!(bs.has(3));
    }

    #[test]
    fn remove_and_returns_old_value() {
        let mut bs = bitvec(16);

        assert_eq!(BitSetMut::remove(&mut bs, 5), Some(false));
        assert!(!bs.has(5));

        assert_eq!(bs.set(5).unwrap(), false);
        assert!(bs.has(5));

        assert_eq!(BitSetMut::remove(&mut bs, 5), Some(true));
        assert!(!bs.has(5));
    }

    #[test]
    fn out_of_bounds_insert_and_remove() {
        let mut bs = bitvec(8);

        // slots.len() == 1, so index 8 => slot_idx 1 (OOB)
        let err = bs.set(8).unwrap_err();
        assert_eq!(err, 1);

        assert_eq!(BitSetMut::remove(&mut bs, 8), None);

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

        let bs = bitvec_from(&input);
        let output: Vec<bool> = bitvec_to_bools(input.len(), &bs);

        assert_eq!(output, input);
    }

    #[test]
    fn cross_byte_boundaries() {
        // boundary at 8 bits
        let mut bs = bitvec(16);

        bs.set(7).unwrap();
        bs.set(8).unwrap();
        bs.set(15).unwrap();

        assert!(bs.has(7));
        assert!(bs.has(8));
        assert!(bs.has(15));

        // byte0: bit7 set => 0b1000_0000
        // byte1: bit0 + bit7 set => 0b1000_0001
        assert_eq!(bs, &[0b1000_0000, 0b1000_0001]);
    }
}
