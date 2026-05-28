import { assert } from "../utils/common.ts";

export * from "./decoder.ts"
export * from "./encoder.ts"
export * from "./varint.ts"
export * from "./zigzag.ts"

export enum DataType {
    False = 0,
    True = 1,

    U8 = 2,
    I8 = 3,

    F32 = 4,
    F64 = 5,

    UInt = 6,
    Int = 7,

    Str = 8,

    Struct = 9,
    StructEnd = 10,

    Union = 11,
    List = 12,
    Table = 13,

    UnknownI = 14,
    UnknownII = 15,
}

export namespace DataType {
    export function fromBool(bool: boolean): DataType {
        return +bool;
    }
    export function fromStr(str: string): DataType {
        let ty = DataType[str as "F32"];
        assert(ty !== undefined, () => `invalid type: ${str}`);
        return ty
    }
}
