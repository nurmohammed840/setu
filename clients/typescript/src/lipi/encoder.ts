import { DataType } from "./mod.ts";
import { encodeVarInt } from "./varint.ts";
import { zigzagEncode } from "./zigzag.ts";
import { bitvecFrom } from "../bitset.ts";
import { Buffer } from "../utils/buffer.ts";
import { assert } from "../utils/common.ts";
import { isLittleEndian } from "../utils/bytes.ts";

const utf8Encoder = new TextEncoder();
const IS_LITTLE_ENDIAN = isLittleEndian()

export class Writer extends Buffer {
    writeF32(num: number) {
        const buf = new ArrayBuffer(4);
        new DataView(buf).setFloat32(0, num, true); // true = little-endian
        this.append(new Uint8Array(buf));
    }

    writeF64(num: number) {
        const buf = new ArrayBuffer(8);
        new DataView(buf).setFloat64(0, num, true); // true = little-endian
        this.append(new Uint8Array(buf));
    }

    writeI8(num: number) {
        assert(num >= -128 && num <= 127, () => `I8 out of range: ${num} (expected -128..=127)`);

        const buf = new ArrayBuffer(1);
        new DataView(buf).setInt8(0, num);
        this.append(new Uint8Array(buf));
    }

    writeUint(num: number | bigint) {
        this.append(encodeVarInt(num));
    }

    writeInt(num: number | bigint) {
        this.writeUint(zigzagEncode(BigInt(num)));
    }

    writeBytes(bytes: Uint8Array) {
        this.writeUint(bytes.length);
        this.append(bytes);
    }

    writeUTF8(text: string) {
        this.writeBytes(utf8Encoder.encode(text));
    }

    write_field_id_and_ty(num: number, ty: DataType) {
        assert(Number.isInteger(num) && num >= 0, () => `expected non-negative integer, got: ${num}`);

        if (num < 15) return this.writeByte((num << 4) | ty);

        this.writeByte((0b1111 << 4) | ty);
        this.writeUint(num - 15)
    }

    write_len_and_ty(num: number, ty: DataType) {
        assert(num <= 0xFF_FF_FF_F); // 28 bits
        this.write_field_id_and_ty(num, ty)
    }
}

// =================================================================

export abstract class Encoder {
    abstract readonly FIELD_TY: DataType;
    abstract encode(w: Writer): void;

    toBytes() {
        let buf = new Writer();
        this.encode(buf);
        return buf.data()
    }
}

export class U8 extends Encoder {
    FIELD_TY = DataType.U8;
    constructor(public val: number) { super(); }
    encode(w: Writer) { w.writeByte(this.val); }
}

export class I8 extends Encoder {
    FIELD_TY = DataType.I8;
    constructor(public val: number) { super(); }
    encode(w: Writer) { w.writeI8(this.val) }
}

export class F32 extends Encoder {
    FIELD_TY = DataType.F32;
    constructor(public val: number) { super() }
    encode(w: Writer) { w.writeF32(this.val) }
}

export class UInt extends Encoder {
    FIELD_TY = DataType.UInt;
    constructor(public val: number | bigint) { super() }
    encode(w: Writer) { w.writeUint(this.val) }
}

export class Int extends Encoder {
    FIELD_TY = DataType.Int;
    constructor(public val: number | bigint) { super() }
    encode(w: Writer) { w.writeInt(this.val) }
}


// ============================ Struct =============================

export type StructTy =
    | { [key: number]: Ty | boolean | undefined; }
    | Array<boolean | Ty | undefined>;

export class Struct extends Encoder {
    FIELD_TY = DataType.Struct;
    constructor(public val: StructTy) { super(); }
    encode(w: Writer) {
        for (let [k, v] of Object.entries(this.val)) {
            let id = +k;
            if (v === undefined) continue;
            if (typeof v == "boolean") {
                w.write_field_id_and_ty(id, DataType.fromBool(v));
                continue;
            }
            w.write_field_id_and_ty(id, fieldTy(v));
            encode(w, v);
        }
        w.writeByte(DataType.StructEnd);
    }
}

export class Field extends Encoder {
    FIELD_TY = DataType.Union;
    constructor(public id: number, public val: boolean | Ty) { super(); }
    encode(w: Writer) {
        if (typeof this.val == "boolean")
            return w.write_field_id_and_ty(this.id, DataType.fromBool(this.val));

        w.write_field_id_and_ty(this.id, fieldTy(this.val));
        encode(w, this.val);
    }
}

// ============================ LIST =============================

type ListOf<T> = T extends unknown ? T[] : never;
export type ListTy = ListOf<Ty> | Set<Ty>;

export class Bools extends Encoder {
    FIELD_TY = DataType.List;
    constructor(public val: boolean[]) { super() }
    encode(w: Writer) {
        w.write_len_and_ty(this.val.length, DataType.True);
        w.append(bitvecFrom(this.val).asBytes())
    }
}

export class List extends Encoder {
    FIELD_TY = DataType.List;
    constructor(public ty: DataType, public val: ListTy) {
        super();
        let { value } = this.val.values().next();
        if (value !== undefined) {
            let vty = fieldTy(value);
            assert(vty == ty, () => `expected list ty ${DataType[ty]}, found: ${DataType[vty]}`);
        }
    }

    encode(w: Writer) {
        let len = this.val instanceof Set ? this.val.size : this.val.length;
        w.write_len_and_ty(len, this.ty);

        for (let v of this.val) encode(w, v)
    }
}

// ============================= TABLE =============================

export class Table extends Encoder {
    FIELD_TY = DataType.Table;
    constructor(public ty: DataType, public val: Map<Ty, Ty>) { super(); }
    encode(w: Writer) { }
}

// =================================================================

export type Ty =
    | Encoder
    | string | number | bigint
    | Uint8Array | Int8Array
    | Float32Array | Float64Array
    | Uint16Array | Uint32Array | BigUint64Array
    | Int16Array | Int32Array | BigInt64Array;

export function fieldTy(v: Ty) {
    if (v instanceof Encoder) return v.FIELD_TY;

    if (typeof v == "string") return DataType.Str;
    if (typeof v == "number") return DataType.F64;
    if (typeof v == "bigint") return DataType.Int;

    if (
        v instanceof Uint8Array || v instanceof Int8Array ||
        v instanceof Float32Array || v instanceof Float64Array ||
        v instanceof Uint16Array || v instanceof Uint32Array || v instanceof BigUint64Array ||
        v instanceof Int16Array || v instanceof Int32Array || v instanceof BigInt64Array
    ) return DataType.List;

    throw new Error(`unknown type: ${typeof v}`);
}

export function encode(w: Writer, v: Ty) {
    if (v instanceof Encoder) return v.encode(w);

    if (typeof v == "string") return w.writeUTF8(v);
    if (typeof v == "number") return w.writeF64(v);
    if (typeof v == "bigint") return w.writeInt(v);


    if (v instanceof Uint8Array) {
        w.write_len_and_ty(v.length, DataType.U8);
        return w.append(v);
    }

    if (v instanceof Int8Array) {
        w.write_len_and_ty(v.length, DataType.I8);
        return w.append(new Uint8Array(v.buffer, v.byteOffset, v.byteLength));
    }

    if (v instanceof Float32Array || v instanceof Float64Array) return encodeFloatArray(w, v)

    if (v instanceof Uint16Array || v instanceof Uint32Array || v instanceof BigUint64Array) {
        w.write_len_and_ty(v.length, DataType.UInt);
        for (let n of v) w.writeUint(n);
        return
    }

    if (v instanceof Int16Array || v instanceof Int32Array || v instanceof BigInt64Array) {
        w.write_len_and_ty(v.length, DataType.Int);
        for (let n of v) w.writeInt(n);
        return
    }

    throw new Error(`unknown value: ${v}`);
}

function encodeFloatArray(w: Writer, nums: Float32Array | Float64Array) {
    let ty = nums instanceof Float32Array ? DataType.F32 : DataType.F64;
    w.write_len_and_ty(nums.length, ty);

    if (IS_LITTLE_ENDIAN) return w.append(new Uint8Array(nums.buffer, nums.byteOffset, nums.byteLength));

    if (ty == DataType.F32)
        for (let n of nums) w.writeF32(n)
    else
        for (let n of nums) w.writeF64(n)
}

// ====================================================

let w = new Writer();
encode(w, new Float64Array([-2]));

let v: Ty = new Struct({
    1: false,
    6: new Struct(["asd"])
});

encode(w, v);
console.log(w.data());

