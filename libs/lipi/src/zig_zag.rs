pub fn zigzag_encode(num: i64) -> u64 {
    ((num << 1) ^ (num >> 63)) as u64
}

pub fn zigzag_decode(num: u64) -> i64 {
    ((num >> 1) as i64) ^ -((num & 1) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_zig_zag(actual: i64, encoded: u64) {
        assert_eq!(zigzag_encode(actual), encoded);
        assert_eq!(zigzag_decode(encoded), actual);
    }

    #[test]
    fn test_zig_zag_encoding() {
        check_zig_zag(0, 0);
        check_zig_zag(-1, 1);
        check_zig_zag(1, 2);
        check_zig_zag(-2, 3);
        check_zig_zag(2, 4);

        check_zig_zag(i64::MIN, u64::MAX);
        check_zig_zag(i64::MAX, u64::MAX - 1);

        check_zig_zag(i32::MIN.into(), u32::MAX.into());
        check_zig_zag(i32::MAX.into(), (u32::MAX - 1).into());
    }
}
