use lipi::Encoder;

#[derive(Encoder)]
struct User {
    #[key = 1]
    name: String,
    #[key = 2]
    email: Option<String>,
    #[key = 15]
    age: u8,
    #[key = 3]
    msg: Vec<u8>,
    #[key = 4]
    balance: f32,
}

#[test]
fn test_name() {
    let user = User {
        name: "John Doe".to_string(),
        email: Some("john_doe@xyz.com".into()),
        age: 42,
        msg: vec![1, 2, 3, 4, 5],
        balance: 4.,
    };

    let mut buf = Vec::new();
    user.encode(&mut buf).unwrap();
    // println!("buf: {:#?}", buf);

    let mut reader = &buf[..];
    let val = lipi::Entries::parse(&mut reader).unwrap();
    println!("{:#?}", lipi::Value::Struct(val));
}
