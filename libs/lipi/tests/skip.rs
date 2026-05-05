#[allow(unused)]
mod data;

use lipi::{Decode, Encode};

#[derive(Decode, Debug)]
enum Ignore {
    Data = 0,
}

impl Ignore {
    fn encode<T: Encode>(data: T) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        lipi::encoder::encode_field_id_and_ty(&mut buf, 0, T::TY).unwrap();

        let raw = data.to_bytes().unwrap();
        buf.extend_from_slice(&raw);

        buf
    }
}

#[test]
fn skip_struct() {
    let data = (data::Types::min(), data::Types::max());
    let mut buf = Ignore::encode(data);

    let msg = b"Hello, World!";
    buf.extend_from_slice(msg);

    let mut encoded = buf.as_slice();
    Ignore::decode(&mut encoded).unwrap();

    assert_eq!(encoded, msg);
}
