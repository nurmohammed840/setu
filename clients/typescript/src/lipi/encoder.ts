import { Buffer } from "../utils/buffer.ts";
import { DataType } from "./mod.ts";
import { encodeVarInt } from "./varint.ts";
import { zigzagEncode } from "./zigzag.ts";

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

// ---------------------------------------------------

abstract class Encoder {
    abstract TY: DataType;
    abstract encode(writer: Writer): void;

    toBytes() {
        let buf = new Writer();
        this.encode(buf);
        return buf.data()
    }
}

// ---------------------------------------------------

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

export class F64 extends Encoder {
    TY = DataType.F64;
    constructor(public val: number) { super() }
    encode(w: Writer) { w.writeF64(this.val) }
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

export class Str extends Encoder {
    TY = DataType.Str;
    constructor(public val: string) { super() }
    encode(w: Writer) { w.writeUTF8(this.val); }
}

