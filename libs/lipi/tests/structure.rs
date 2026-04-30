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

#[test]
fn test_basic_struct() {
    let user = User {
        id: 42,
        name: "Nur".into(),
        email: None,
    };

    check_msg(&user, &[0x3, 0x6, 0x2A, 0x18, 0x3, 0x4E, 0x75, 0x72]);
}

fn check_msg<T: Encode>(msg: &T, raw: &[u8]) {
    let mut buf = vec![];
    msg.encode(&mut buf).unwrap();
    assert_eq!(buf, raw);
}
