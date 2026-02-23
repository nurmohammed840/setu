mod entries;

// #[derive(Clone)]
// pub enum Value<'de> {
//     Bool(bool),

//     U8(u8),
//     I8(i8),

//     F32(f32),
//     F64(f64),

//     UInt(u64),
//     Int(i64),

//     Str(&'de str),

//     Struct(Entries<'de>),
//     Union(Box<Entry<'de>>),
//     List(List<'de>),
//     Table(Table<'de>),

//     // ---------------
//     UnknownI(&'de [u8]),
//     UnknownII(&'de [u8]),
//     UnknownIII(&'de [u8]),
// }