use std::slice;

use crate::{Result, errors};

pub fn read_byte(buf: &mut &[u8]) -> Result<u8> {
    if buf.is_empty() {
        return Err(errors::UnexpectedEof { size: 1 }.into());
    }
    unsafe {
        let byte = *buf.get_unchecked(0);
        *buf = buf.get_unchecked(1..);
        Ok(byte)
    }
}

pub fn read_buf<const N: usize>(reader: &mut &[u8]) -> Result<[u8; N]> {
    read_bytes(reader, N).map(|bytes| bytes.try_into().unwrap())
}

pub fn read_bytes<'de>(reader: &mut &'de [u8], len: usize) -> Result<&'de [u8]> {
    if len > reader.len() {
        return Err(errors::UnexpectedEof { size: len }.into());
    }
    unsafe {
        let slice = reader.get_unchecked(..len);
        *reader = reader.get_unchecked(len..);
        Ok(slice)
    }
}

pub fn i8_slice_from(bytes: &[u8]) -> &[i8] {
    unsafe { slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len()) }
}

pub fn u8_slice_from(data: &[i8]) -> &[u8] {
    unsafe { slice::from_raw_parts(data.as_ptr().cast(), data.len()) }
}

pub fn bool_from(byte: u8) -> Result<bool> {
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        v => Err(errors::ParseError::new(format!("invalid boolean value: `{v}`")).into()),
    }
}

pub fn try_convert_into_vec<T, M>(items: &[T]) -> Result<Vec<M>, M::Error>
where
    T: Clone,
    M: TryFrom<T>,
{
    try_convert_vec_from(items, |t| M::try_from(t.clone()))
}

pub fn try_convert_vec_from<'a, T, M, E>(
    items: &'a [T],
    f: fn(&'a T) -> Result<M, E>,
) -> Result<Vec<M>, E> {
    let mut arr = Vec::<M>::with_capacity(items.len());
    for idx in 0..items.len() {
        unsafe {
            let val = f(items.get_unchecked(idx))?;
            arr.as_mut_ptr().add(idx).write(val);
            arr.set_len(idx + 1);
        }
    }
    Ok(arr)
}

pub fn try_collect<T, E>(len: usize, mut f: impl FnMut() -> Result<T, E>) -> Result<Vec<T>, E> {
    let mut arr = Vec::<T>::with_capacity(len);
    for idx in 0..len {
        unsafe {
            let val = f()?;
            arr.as_mut_ptr().add(idx).write(val);
            arr.set_len(idx + 1);
        }
    }
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_byte() {
        let mut buf = [1; 1].as_slice();
        assert_eq!(read_byte(&mut buf).unwrap(), 1);

        assert_eq!(buf, &[]);
        assert!(read_byte(&mut buf).is_err());
    }

    #[test]
    fn test_read_bytes() {
        let mut buf = [1, 2, 3].as_slice();
        assert_eq!(read_bytes(&mut buf, 2).unwrap(), &[1, 2]);

        assert_eq!(buf, &[3]);
        assert!(read_bytes(&mut buf, 2).is_err());

        assert_eq!(buf, &[3]);
        assert_eq!(read_bytes(&mut buf, 0).unwrap(), &[]);
        assert_eq!(read_bytes(&mut buf, 1).unwrap(), &[3]);

        assert_eq!(buf, &[]);
        assert!(read_bytes(&mut buf, 1).is_err());
    }

    #[test]
    fn test_conversion_u8_to_i8_slice() {
        let bytes: &[u8] = &[0, 1, 127, 128, 254, 255];
        let i8_slice = i8_slice_from(bytes);

        assert_eq!(i8_slice, [0, 1, 127, -128, -2, -1]);
        assert_eq!(u8_slice_from(i8_slice), bytes);
    }

    #[test]
    fn test_try_collect() {
        let mut i = 0;
        let items = try_collect::<_, ()>(3, || {
            i += 1;
            Ok(i.to_string())
        })
        .unwrap();

        let values = try_convert_vec_from(&items, |s| s.parse::<i32>()).unwrap();
        assert_eq!(values, [1, 2, 3]);
    }
}
