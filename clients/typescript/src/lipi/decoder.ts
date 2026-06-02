import { Bytes } from "../utils/bytes.ts";
import { assert, IS_LITTLE_ENDIAN } from "../utils/common.ts";
import { decodeVarInt } from "./varint.ts";
import { DataType } from "./type.ts";
import { zigzagDecode } from "./zigzag.ts";
import { bitvecToBools, boolPackedLen } from "../bitset.ts";

const { check } = DataType;
const UTF8_DECODER = new TextDecoder();

type Decoder<T> = (this: Decode) => T;
export class Deserialize {
    constructor(public buf: Bytes) { }
    read_uint() {
        return decodeVarInt(this.buf);
    }

    read_bytes() {
        let len = Number(this.read_uint());
        return this.buf.take(len);
    }

    read_field_id_and_ty() {
        let byte = this.buf.nextByte();

        let ty = byte & 0b1111;
        let id = byte >> 4;

        if (id === 0b1111) {
            id = Number(this.read_uint()) + 15;
        }

        return [id, ty as DataType] as const;
    }

    read_len_and_ty() {
        return this.read_field_id_and_ty();
    }
}

export class Decode extends Deserialize {
    U8() {
        return this.buf.nextByte();
    }

    I8() {
        return view(this.buf.take(1)).getInt8(0);
    }

    F32() {
        return view(this.buf.take(4)).getFloat32(0, true); // true = little-endian
    }

    F64() {
        return view(this.buf.take(8)).getFloat64(0, true); // true = little-endian
    }

    Uint() {
        return this.read_uint();
    }

    Int() {
        return zigzagDecode(this.read_uint())
    }

    Str() {
        return UTF8_DECODER.decode(this.read_bytes());
    }

    List<T>(f: Decoder<T>) {
        let self = this;
        return function List(): Array<T> {
            let [length, ty] = self.read_len_and_ty();
            checkDecoder(f, ty);
            return Array.from({ length }, () => f.call(self));
        }
    }

    ListU8() {
        let [len, ty] = this.read_len_and_ty();
        check(DataType.U8, ty);
        return this.buf.take(len);
    }

    ListI8() {
        let [len, ty] = this.read_len_and_ty();
        check(DataType.I8, ty);

        let buf = this.buf.take(len);
        return new Int8Array(buf.buffer, buf.byteOffset, len);
    }

    ListF32() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.F32, ty);

        if (IS_LITTLE_ENDIAN) {
            let buf = this.buf.take(length * 4);
            return new Float32Array(buf.buffer, buf.byteOffset, length);
        }

        return Float32Array.from({ length }, () => this.F32());
    }

    ListF64() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.F32, ty);

        if (IS_LITTLE_ENDIAN) {
            let buf = this.buf.take(length * 8);
            return new Float64Array(buf.buffer, buf.byteOffset, length);
        }

        return Float64Array.from({ length }, () => this.F64());
    }

    ListU16() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.UInt, ty);
        return Uint16Array.from({ length }, () => Number(this.Uint()));
    }

    ListU32() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.UInt, ty);
        return Uint32Array.from({ length }, () => Number(this.Uint()));
    }

    ListU64() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.UInt, ty);
        return BigUint64Array.from({ length }, () => this.Uint());
    }

    ListI16() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.Int, ty);
        return Int16Array.from({ length }, () => Number(this.Int()));
    }

    ListI32() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.Int, ty);
        return Int32Array.from({ length }, () => Number(this.Int()));
    }

    ListI64() {
        let [length, ty] = this.read_len_and_ty();
        check(DataType.Int, ty);
        return BigInt64Array.from({ length }, () => this.Int());
    }

    ListBool() {
        let [count, ty] = this.read_len_and_ty();
        check(DataType.True, ty);
        const bytes = this.buf.take(boolPackedLen(count));
        return bitvecToBools(count, bytes);
    }

    Table<K, V>(k: Decoder<K>, v: Decoder<V>) {
        let self = this;
        return function Table(): Map<K, V> {
            let columnCount = Number(self.read_uint());
            let length = Number(self.read_uint());

            assert(columnCount == 2, RangeError, () => `invalid column count: ${columnCount}`);

            let [field, ty] = self.read_field_id_and_ty();
            if (field == 0) {
                checkDecoder(k, ty);
                let keys = Array.from({ length }, () => k.call(self));

                let [field2, ty2] = self.read_field_id_and_ty();
                assert(field2 == 1, TypeError, () => `invalid column (value) id: expected \`1\`, found ${field2}`);
                checkDecoder(v, ty2);

                let map = new Map();
                for (let key of keys) map.set(key, v.call(self))
                return map;
            }
            if (field == 1) {
                checkDecoder(v, ty);
                let values = Array.from({ length }, () => v.call(self));

                let [field2, ty2] = self.read_field_id_and_ty();
                assert(field2 === 0, TypeError, () => `invalid column (key) id: expected \`0\`, found ${field2}`);
                checkDecoder(k, ty2);

                let map = new Map();
                for (let val of values) map.set(k.call(self), val);
                return map;
            }

            throw new TypeError(`invalid column id: expected \`0\` or \`1\`, found ${field}`);
        }
    }
}

// ========================================================

function view(data: Uint8Array) {
    return new DataView(
        data.buffer,
        data.byteOffset,
        data.byteLength
    )
}

function checkDecoder<T>(f: Decoder<T>, ty: DataType) {
    check(DataType.fromStr(f.name), ty);
}
