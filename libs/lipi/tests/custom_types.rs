use lipi::{ConvertFrom, Decoder, Encoder, IntoValue, Value, errors::ConvertError};

struct Bytes<'de> {
    data: &'de [u8],
}

impl<'de> IntoValue<'de> for Bytes<'de> {
    fn to_value(&self) -> Value<'de> {
        Value::Bytes(self.data)
    }
}

impl<'de> ConvertFrom<&Value<'de>> for Bytes<'de> {
    fn convert_from(value: &Value<'de>) -> Result<Self, ConvertError> {
        match value {
            Value::Bytes(data) => Ok(Bytes { data }),
            _ => Err(ConvertError::from("expected `Bytes` type")),
        }
    }
}

#[derive(Encoder, Decoder)]
struct CustomType<'de> {
    #[key = 1]
    bytes: Bytes<'de>,
}

#[test]
fn test_custom_type() {
    let bytes = CustomType {
        bytes: Bytes {
            data: b"hello world",
        },
    };

    let mut buf = Vec::new();
    bytes.encode(&mut buf).unwrap();
    // println!("reader: {:#?}", buf);

    let mut reader = &buf[..];
    let custom_type = CustomType::parse(&mut reader).unwrap();
    assert_eq!(custom_type.bytes.data, b"hello world");
}
