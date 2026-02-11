// use std::io::{self, Write};

// use lipi::{ConvertFrom, Decoder, Encoder, FieldEncoder, List, Value, errors::ConvertError};

// struct Bytes<'de> {
//     data: &'de [u8],
// }

// impl<'de> FieldEncoder for Bytes<'de> {
//     fn encode(&self, writer: &mut dyn Write, id: u16) -> io::Result<()> {
//         Value::encode(&Value::List(List::U8(self.data)), writer, id)
//     }
// }

// impl<'de> ConvertFrom<&Value<'de>> for Bytes<'de> {
//     fn convert_from(value: &Value<'de>) -> Result<Self, ConvertError> {
//         match value {
//             Value::List(List::U8(data)) => Ok(Bytes { data }),
//             _ => Err(ConvertError::from("expected `Bytes` type")),
//         }
//     }
// }

// #[derive(Encoder, Decoder)]
// struct CustomType<'de> {
//     #[key = 1]
//     bytes: Bytes<'de>,
// }

// #[test]
// fn test_custom_type() {
//     let bytes = CustomType {
//         bytes: Bytes {
//             data: b"hello world",
//         },
//     };

//     let mut buf = Vec::new();
//     Encoder::encode(&bytes, &mut buf).unwrap();
//     // println!("reader: {:#?}", buf);

//     let mut reader = &buf[..];
//     let custom_type = CustomType::parse(&mut reader).unwrap();
//     assert_eq!(custom_type.bytes.data, b"hello world");
// }
