pub fn into_u64(num: i64) -> u64 {
    ((num << 1) ^ (num >> 63)) as u64
}

pub fn from_u64(num: u64) -> i64 {
    ((num >> 1) as i64) ^ -((num & 1) as i64)
}

#[test]
#[cfg(test)]
fn test_zig_zag_encoding() {
    assert_eq!(into_u64(0), 0);
    assert_eq!(into_u64(-1), 1);
    assert_eq!(into_u64(1), 2);
    assert_eq!(into_u64(-2), 3);
    assert_eq!(into_u64(2), 4);

    assert_eq!(from_u64(0), 0);
    assert_eq!(from_u64(1), -1);
    assert_eq!(from_u64(2), 1);
    assert_eq!(from_u64(3), -2);
    assert_eq!(from_u64(4), 2);

    assert_eq!(into_u64(i64::MIN), u64::MAX);
    assert_eq!(into_u64(i64::MAX), u64::MAX - 1);

    assert_eq!(from_u64(u64::MAX), i64::MIN);
    assert_eq!(from_u64(u64::MAX - 1), i64::MAX);
}
