use crate::*;

ty! {
    u8 = Type::U8
    u16 = Type::U16
    u32 = Type::U32
    u64 = Type::U64
    u128 = Type::U128

    i8 = Type::I8
    i16 = Type::I16
    i32 = Type::I32
    i64 = Type::I64
    i128 = Type::I128

    f32 = Type::F32
    f64 = Type::F64
}

ty! {
    usize: (_) {
        match Self::BITS {
            32 => Type::U32,
            64 => Type::U64,
            _ => Type::U16,
        }
    }

    isize: (_) {
        match Self::BITS {
            32 => Type::I32,
            64 => Type::I64,
            _ => Type::I16,
        }
    }
}
