import { assert } from "../utils/common.ts";

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

    export function asBool(ty: DataType) {
        assert(ty == DataType.False || ty == DataType.True, TypeError, () => `expected: False or True, found: ${DataType[ty]}`);
        return ty == DataType.True;
    }

    export function fromStr(str: string): DataType {
        let ty = DataType[str as "F32"];
        assert(ty !== undefined, TypeError, () => `invalid type: ${str}`);
        return ty
    }

    export function expected(expected: DataType, found: DataType) {
        assert(expected == found, TypeError, () => `expected: ${DataType[expected]}, found: ${DataType[found]}`);
    }
}
