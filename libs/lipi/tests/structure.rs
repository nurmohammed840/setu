use lipi::*;

#[derive(Encode, /* Decode, */ Clone, Debug, PartialEq)]
pub struct User {
    #[key = 0]
    id: u32,

    #[key = 1]
    name: String,

    #[key = 2]
    email: Option<String>,
}

impl<'de> lipi::Decode<'de> for User {
    const TY: lipi::DataType = DataType::Struct;

    fn decode(reader: &mut &'de [u8]) -> Result<Self> {
        let mut name: Option<_> = None;
        let mut id: Option<_> = None;
        let mut email: Option<String> = None;

        let mut fd = decoder::FieldInfoDecoder::new(reader)?;

        println!("fd.len(): {:#?}", fd.len());

        while let Some((key, ty)) = fd.next_field_id_and_ty()? {
            println!("(key, ty): {:#?}", (key, ty));
            match key {
                0 => id = fd.decode(ty, "id")?,
                1 => name = fd.decode(ty, "name")?,
                2 => email = fd.decode(ty, "email")?,
                // Unknown Field
                _ => {}
            }
        }

        println!(" ========= ");

        Ok(Self {
            id: decoder::Optional::convert(id, "id")?,
            name: decoder::Optional::convert(name, "name")?,
            email: decoder::Optional::convert(email, "email")?,
        })
    }
}

#[test]
fn test_basic_struct() {
    let user = User {
        id: 42,
        name: "Nur".into(),
        email: None,
    };

    let user = check_msg(&user, &[0x3, 0x6, 0x2A, 0x18, 0x3, 0x4E, 0x75, 0x72]);

    println!("{:#?}", user);
}

// fn check_msg<T: Encode>(msg: &T, raw: &[u8]) {
//     let mut buf = vec![];
//     msg.encode(&mut buf).unwrap();
//     assert_eq!(buf, raw);
// }

fn check_msg<T: Encode + for<'a> Decode<'a>>(msg: &T, raw: &[u8]) -> T {
    let mut buf = vec![];
    msg.encode(&mut buf).unwrap();
    assert_eq!(buf, raw);

    T::decode(&mut buf.as_slice()).unwrap()
}
