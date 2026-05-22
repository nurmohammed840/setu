import { Buffer } from "../utils/buffer.ts";
import { DataType } from "./mod.ts";
import { encodeVarInt } from "./varint.ts";
import { zigzagEncode } from "./zigzag.ts";
import { bitvecFrom } from "../bitset.ts";

const utf8Encoder = new TextEncoder();

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
        if (num < -128 || num > 127)
            throw new Error(`I8 out of range: ${num} (expected -128..127)`);

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
        if (num < 15) return this.writeByte((num << 4) | ty);

        this.writeByte((0b1111 << 4) | ty);
        this.writeUint(num - 15)
    }
}

// =================================================================

export abstract class Encoder {
    abstract readonly TY: DataType;
    abstract encode(writer: Writer): void;

    toBytes() {
        let buf = new Writer();
        this.encode(buf);
        return buf.data()
    }
}

export class U8 extends Encoder {
    TY = DataType.U8;
    constructor(public val: number) { super(); }
    encode(w: Writer) { w.writeByte(this.val); }
}

export class I8 extends Encoder {
    TY = DataType.I8;
    constructor(public val: number) { super(); }
    encode(w: Writer) { w.writeI8(this.val) }
}

export class F32 extends Encoder {
    TY = DataType.F32;
    constructor(public val: number) { super() }
    encode(w: Writer) { w.writeF32(this.val) }
}

export class UInt extends Encoder {
    TY = DataType.UInt;
    constructor(public val: number | bigint) { super() }
    encode(w: Writer) { w.writeUint(this.val) }
}

export class Int extends Encoder {
    TY = DataType.Int;
    constructor(public val: number | bigint) { super() }
    encode(w: Writer) { w.writeInt(this.val) }
}

export class Struct extends Encoder {
    TY = DataType.Struct;
    constructor(public val: StructTy) { super() }
    encode(w: Writer) {
        for (let [k, v] of Object.entries(this.val)) {
            let id = +k;
            if (v === undefined) continue;
            if (typeof v == "boolean") {
                w.write_field_id_and_ty(id, DataType.fromBool(v));
                continue;
            }
            w.write_field_id_and_ty(id, dataTy(v));
            encode(w, v);
        }
        w.writeByte(DataType.StructEnd);
    }
}

// ============================ LIST =============================

export class Bools extends Encoder {
    TY = DataType.List;
    constructor(public val: boolean[]) { super() }
    encode(w: Writer) {
        w.write_field_id_and_ty(this.val.length, DataType.True);
        w.append(bitvecFrom(this.val).asBytes())
    }
}

// =================================================================

export type Ty =
    | Encoder
    | string | number | bigint
    | Uint8Array
    | StructTy;

export type StructTy = { [key: number]: Ty | boolean | undefined; };

export function dataTy(val: Ty) {
    if (val instanceof Encoder) return val.TY;

    if (typeof val == "string") return DataType.Str;
    if (typeof val == "number") return DataType.F64;
    if (typeof val == "bigint") return DataType.Int;

    if (val instanceof Uint8Array) return DataType.List;

    if (typeof val == "object") return DataType.Struct;

    throw new Error(`unknown type: ${typeof val}`);
}

export function encode(w: Writer, val: Ty) {
    if (val instanceof Encoder) return val.encode(w);

    if (typeof val == "string") return w.writeUTF8(val);
    if (typeof val == "number") return w.writeF64(val);
    if (typeof val == "bigint") return w.writeInt(val);

    if (val instanceof Uint8Array) return w.writeBytes(val);

    if (typeof val == "object") return new Struct(val).encode(w);

    throw new Error(`unknown value: ${val}`);
}


// =================================================================

let v: Ty = {
    1: false,
    6: {
        5: "asd"
    }
};

let w = new Writer();
encode(w, v);
console.log(w.data());
console.log(JSON.stringify(v).length);

