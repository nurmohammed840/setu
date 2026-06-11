import { boolPackedLen } from "../bitset.ts";
import { assert } from "../utils/common.ts";
import { Deserialize, } from "./decoder.ts";
import { DataType } from "./type.ts";

export function skip_field(self: Deserialize, id: number, ty: DataType) {
    try {
        skip_field_value(self, ty);
    } catch (error) {
        throw new Error(`failed to skip field ${id} of type ${DataType[ty]}: ${error}`);
    }
}

export function skip_field_value(self: Deserialize, ty: DataType) {
    switch (ty) {
        case DataType.False: case DataType.True: break;

        case DataType.U8: case DataType.I8: self.buf.nextByte(); break;
        case DataType.F32: self.buf.take(4); break;
        case DataType.F64: self.buf.take(8); break;
        case DataType.UInt: case DataType.Int: self.read_varint(); break;

        case DataType.Str: case DataType.UnknownI: case DataType.UnknownII:
            self.read_bytes(); break;

        case DataType.StructEnd: throw new Error("unexpected StructEnd");
        case DataType.Struct: skip_struct(self); break;
        case DataType.Union: skip_union(self); break;
        case DataType.List: skip_list(self); break;
        case DataType.Table: skip_table(self); break;

        default: throw new Error(`unknown data type: ${ty}`);
    }
}

export function skip_struct(self: Deserialize) {
    let header;
    while (header = self.next_field_id_and_ty()) {
        let [id, ty] = header;
        skip_field(self, id, ty);
    }
}

export function skip_union(self: Deserialize) {
    let [id, ty] = self.read_field_id_and_ty();
    skip_field(self, id, ty);
}

export function skip_list(self: Deserialize) {
    let [len, ty] = self.read_len_and_ty();
    skip_list_values(self, len, ty);
}

export function skip_list_values(self: Deserialize, len: number, ty: DataType) {
    assert(len >= 0, RangeError, () => `invalid list length: ${len}`);
    switch (ty) {
        case DataType.False: throw new Error("unexpected bool packed in list");
        // Ignore packed_bools
        case DataType.True: self.buf.take(boolPackedLen(len)); break;

        case DataType.U8: case DataType.I8: self.buf.take(len); break;
        case DataType.F32: self.buf.take(len * 4); break;
        case DataType.F64: self.buf.take(len * 8); break;
        case DataType.UInt: case DataType.Int:
            while (len--) self.read_varint(); break;

        case DataType.Str: case DataType.UnknownI: case DataType.UnknownII:
            while (len--) self.read_bytes(); break;

        case DataType.StructEnd: throw new Error("unexpected StructEnd");

        case DataType.Struct: while (len--) skip_struct(self); break;
        case DataType.Union: while (len--) skip_union(self); break;
        case DataType.List: while (len--) skip_list(self); break;
        case DataType.Table: while (len--) skip_table(self); break;

        default: throw new Error(`unknown data type: ${ty}`);
    }
}


export function skip_table(self: Deserialize) {
    let cols = self.read_len();
    let len = self.read_len();
    for (let i = 0; i < cols; i++) {
        let [col_id, col_ty] = self.read_field_id_and_ty();
        try {
            skip_list_values(self, len, col_ty);
        } catch (error) {
            throw new Error(`failed to skip column ${col_id} of type ${DataType[col_ty]}: ${error}`);
        }
    }
}
