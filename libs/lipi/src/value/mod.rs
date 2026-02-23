mod entries;
mod list;
mod table;

#[derive(Clone)]
pub enum Value {
    Bool(bool),

    U8(u8),
    I8(i8),

    F32(f32),
    F64(f64),

    UInt(u64),
    Int(i64),

    Str(Box<str>),

    // Struct(Entries<'de>),
    // Union(Box<Entry<'de>>),
    // List(List<'de>),
    // Table(Table<'de>),

    // ---------------
    
    UnknownI(Box<[u8]>),
    UnknownII(Box<[u8]>),
    UnknownIII(Box<[u8]>),
}