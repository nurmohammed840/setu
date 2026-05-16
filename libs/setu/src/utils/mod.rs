pub const fn _concet_bytes<const N: usize>(a: &[u8], b: &[u8]) -> [u8; N] {
    let mut buf = [0; N];

    let mut i = 0;
    while i < a.len() {
        buf[i] = a[i];
        i += 1;
    }

    let mut j = 0;
    while j < b.len() {
        buf[a.len() + j] = b[j];
        j += 1;
    }

    buf
}
