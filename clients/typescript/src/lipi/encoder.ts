import { DataType } from "./type.ts";
import { encodeVarInt } from "./varint.ts";
import { zigzagEncode } from "./zigzag.ts";
import { Buffer } from "../utils/buffer.ts";
import { assert, checkOverflowInt, checkOverflowUint, IS_LITTLE_ENDIAN } from "../utils/common.ts";
import { bitvecFrom } from "../bitset.ts";

const UTF8_ENCODER = new TextEncoder();

type Encoder<T> = (this: Encode, val: T) => void;

export class Writer extends Buffer {
    writeVarint(num: number | bigint) {
        this.append(encodeVarInt(num));
    }

    writeBytes(bytes: Uint8Array) {
        this.writeVarint(bytes.length);
        this.append(bytes);
    }

    write_field_id_and_ty(num: number, ty: DataType) {
        assert(Number.isInteger(num) && num >= 0, RangeError, () => `expected non-negative integer, got: ${num}`);

        if (num < 15) return this.writeByte((num << 4) | ty);

        this.writeByte((0b1111 << 4) | ty);
        this.writeVarint(num - 15)
    }

    write_len_and_ty(num: number, ty: DataType) {
        assert(num <= 0xFF_FF_FF_F); // 28 bits
        this.write_field_id_and_ty(num, ty)
    }
}

export class Encode extends Writer {
    U8(num: number) {
        this.writeByte(num)
    }

    I8(num: number) {
        checkOverflowInt(num, 8);

        const buf = new ArrayBuffer(1);
        new DataView(buf).setInt8(0, num);
        this.append(new Uint8Array(buf));
    }

    F32(num: number) {
        const buf = new ArrayBuffer(4);
        new DataView(buf).setFloat32(0, num, true); // true = little-endian
        this.append(new Uint8Array(buf));
    }

    F64(num: number) {
        const buf = new ArrayBuffer(8);
        new DataView(buf).setFloat64(0, num, true); // true = little-endian
        this.append(new Uint8Array(buf));
    }

    U16 = function Uint(this: Encode, num: number) {
        this.writeVarint(checkOverflowUint(num, 16))
    }

    U32 = function Uint(this: Encode, num: number) {
        this.writeVarint(checkOverflowUint(num, 32))
    }

    U64 = function Uint(this: Encode, num: number | bigint) {
        this.writeVarint(num)
    }

    I16 = function Int(this: Encode, num: number) {
        this.I64(checkOverflowInt(num, 16))
    }

    I32 = function Int(this: Encode, num: number) {
        this.I64(checkOverflowInt(num, 32))
    }

    I64 = function Int(this: Encode, num: number | bigint) {
        this.writeVarint(zigzagEncode(BigInt(num)));
    }

    Str(text: string) {
        this.writeBytes(UTF8_ENCODER.encode(text));
    }

    List<T>(f: Encoder<T>) {
        let self = this;
        return function List(vals: Iterable<T> & { length: number }) {
            self.write_len_and_ty(vals.length, DataType.fromStr(f.name));
            for (let v of vals) f.call(self, v);
        }
    }

    ListU8 = function List(this: Encode, v: Uint8Array) {
        this.write_len_and_ty(v.length, DataType.U8);
        this.append(v)
    }

    ListI8 = function List(this: Encode, v: Int8Array) {
        this.write_len_and_ty(v.length, DataType.I8);
        this.append(RawBytes(v))
    }

    ListF32 = function List(this: Encode, v: Float32Array) {
        this.write_len_and_ty(v.length, DataType.F32);
        if (IS_LITTLE_ENDIAN) return this.append(RawBytes(v));
        for (let n of v) this.F32(n)
    }

    ListF64 = function List(this: Encode, v: Float64Array) {
        this.write_len_and_ty(v.length, DataType.F64);
        if (IS_LITTLE_ENDIAN) return this.append(RawBytes(v));
        for (let n of v) this.F64(n)
    }

    ListU16: (this: Encode, v: Uint16Array) => void = this.List(this.U16);
    ListU32: (this: Encode, v: Uint32Array) => void = this.List(this.U32);
    ListU64: (this: Encode, v: BigUint64Array) => void = this.List(this.U64);

    ListI16: (this: Encode, v: Int16Array) => void = this.List(this.I16);
    ListI32: (this: Encode, v: Int32Array) => void = this.List(this.I32);
    ListI64: (this: Encode, v: BigInt64Array) => void = this.List(this.I64);

    ListBool = function List(this: Encode, bools: Array<boolean>) {
        this.write_len_and_ty(bools.length, DataType.True);
        this.append(bitvecFrom(bools).asBytes())
    }

    Table<K, V>(k: Encoder<K>, v: Encoder<V>) {
        let self = this;
        return function Table(map: Map<K, V>) {
            self.writeVarint(2); // Column count
            self.writeVarint(map.size); // Row count

            self.write_field_id_and_ty(0, DataType.fromStr(k.name));
            for (let key of map.keys()) k.call(self, key);

            self.write_field_id_and_ty(1, DataType.fromStr(v.name));
            for (let val of map.values()) v.call(self, val);
        }
    }
}

export class StructEncoder {
    constructor(public e: Encode) { }

    Field<T>(f: Encoder<T>) {
        return (id: number, v: T) => {
            this.e.write_field_id_and_ty(id, DataType.fromStr(f.name));
            f.call(this.e, v);
        }
    }

    Option<T>(f: (this: this, id: number, value: T) => void) {
        return (id: number, val?: T) => {
            if (val === undefined) return;
            f.call(this, id, val);
        }
    }

    Bool(id: number, bool: boolean) {
        this.e.write_field_id_and_ty(id, DataType.fromBool(bool));
    }

    get U8() {
        return this.Field(this.e.U8)
    }

    get I8() {
        return this.Field(this.e.I8)
    }

    get F32() {
        return this.Field(this.e.F32)
    }

    get F64() {
        return this.Field(this.e.F64)
    }

    get UInt() {
        return this.Field(this.e.U64)
    }

    get Int() {
        return this.Field(this.e.I64)
    }

    get Str() {
        return this.Field(this.e.Str)
    }

    // ===================================

    List<T>(f: Encoder<T>) {
        return this.Field(this.e.List(f))
    }

    get ListU8() {
        return this.Field(this.e.ListU8)
    }

    get ListI8() {
        return this.Field(this.e.ListI8)
    }

    get ListF32() {
        return this.Field(this.e.ListF32)
    }

    get ListF64() {
        return this.Field(this.e.ListF64)
    }

    get ListUint() {
        return this.List(this.e.U64)
    }

    get ListInt() {
        return this.List(this.e.I64)
    }

    get ListStr() {
        return this.List(this.e.Str)
    }

    get ListBool() {
        return this.Field(this.e.ListBool)
    }

    // ===================================

    Table<K, V>(k: Encoder<K>, v: Encoder<V>) {
        return this.Field(this.e.Table(k, v))
    }

    // ===================================

    end() {
        this.e.writeByte(DataType.StructEnd);
    }
}

// ===========================================================

function RawBytes(v: ArrayBufferView) {
    return new Uint8Array(v.buffer, v.byteOffset, v.byteLength)
}

// ===========================================================