pub fn into(num: i64) -> u64 {
    ((num << 1) ^ (num >> 63)) as u64
}

pub fn from(num: u64) -> i64 {
    ((num >> 1) as i64) ^ -((num & 1) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zig_zag() {
        assert_eq!(into(0), 0);
        assert_eq!(into(1), 2);
        assert_eq!(into(-1), 1);
        assert_eq!(into(2), 4);
        assert_eq!(into(-2), 3);

        assert_eq!(from(0), 0);
        assert_eq!(from(2), 1);
        assert_eq!(from(1), -1);
        assert_eq!(from(4), 2);
        assert_eq!(from(3), -2);
    }

    #[test]
    fn test_min_max() {
        assert!(into(i64::MIN) == u64::MAX);
        assert!(from(u64::MAX) == i64::MIN);

        assert!(into(i64::MAX) == u64::MAX - 1);
        assert!(from(u64::MAX - 1) == i64::MAX);
    }
}
