import { Bytes } from "../utils/bytes.ts";
import { assert, IS_LITTLE_ENDIAN, checkOverflowInt, checkOverflowUint } from "../utils/common.ts";
import { decodeVarInt } from "./varint.ts";
import { DataType } from "./type.ts";
import { zigzagDecode } from "./zigzag.ts";
import { bitvecToBools, boolPackedLen } from "../bitset.ts";

const { expected } = DataType;
const UTF8_DECODER = new TextDecoder();

export type Decoder<T> = (this: Decode) => T;
export class Deserialize {
    constructor(public buf: Bytes) { }
    read_varint() {
        return decodeVarInt(this.buf);
    }

    read_len() {
        return Number(this.read_varint())
    }

    read_bytes() {
        let len = this.read_len();
        return this.buf.take(len);
    }

    read_field_id_and_ty() {
        let byte = this.buf.nextByte();

        let ty = byte & 0b1111;
        let id = byte >> 4;

        if (id == 0b1111) {
            id = Number(this.read_varint()) + 15;
        }

        return [id, ty as DataType] as const;
    }

    read_len_and_ty() {
        return this.read_field_id_and_ty();
    }
}

export class Decode extends Deserialize {
    Bool(): boolean {
        throw "unreachable"
    }

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

    U16 = function Uint(this: Decode) {
        return checkOverflowUint(Number(this.read_varint()), 16);
    }

    U32 = function Uint(this: Decode) {
        return checkOverflowUint(Number(this.read_varint()), 32);
    }

    U64 = function Uint(this: Decode) {
        return this.read_varint();
    }

    I16 = function Int(this: Decode) {
        return checkOverflowInt(Number(this.I64()), 16);
    }

    I32 = function Int(this: Decode) {
        return checkOverflowInt(Number(this.I64()), 32);
    }

    I64 = function Int(this: Decode) {
        return zigzagDecode(this.read_varint())
    }

    Str() {
        return UTF8_DECODER.decode(this.read_bytes());
    }

    List<T>(f: Decoder<T>) {
        let self = this;
        return function List(): Array<T> {
            let [length, ty] = self.read_len_and_ty();
            expectedTy(f, ty);
            return Array.from({ length }, () => f.call(self));
        }
    }

    ListU8 = function List(this: Decode) {
        let [len, ty] = this.read_len_and_ty();
        expected(DataType.U8, ty);
        return this.buf.take(len);
    }

    ListI8 = function List(this: Decode) {
        let [len, ty] = this.read_len_and_ty();
        expected(DataType.I8, ty);

        let buf = this.buf.take(len);
        return new Int8Array(buf.buffer, buf.byteOffset, len);
    }

    ListF32 = function List(this: Decode) {
        let [length, ty] = this.read_len_and_ty();
        expected(DataType.F32, ty);

        if (IS_LITTLE_ENDIAN) {
            let buf = this.buf.take(length * 4);
            return new Float32Array(buf.buffer, buf.byteOffset, length);
        }

        return Float32Array.from({ length }, () => this.F32());
    }

    ListF64 = function List(this: Decode) {
        let [length, ty] = this.read_len_and_ty();
        expected(DataType.F64, ty);

        if (IS_LITTLE_ENDIAN) {
            let buf = this.buf.take(length * 8);
            return new Float64Array(buf.buffer, buf.byteOffset, length);
        }

        return Float64Array.from({ length }, () => this.F64());
    }

    ListU16 = ListType(this.U16, Uint16Array.from)
    ListU32 = ListType(this.U32, Uint32Array.from)
    ListU64 = ListType(this.U64, BigUint64Array.from)

    ListI16 = ListType(this.I16, Int16Array.from)
    ListI32 = ListType(this.I32, Int32Array.from)
    ListI64 = ListType(this.I64, BigInt64Array.from)

    ListBool = function List(this: Decode) {
        let [len, ty] = this.read_len_and_ty();
        expected(DataType.True, ty);

        const bitvec = this.buf.take(boolPackedLen(len));
        return bitvecToBools(bitvec, len);
    }

    Table<K, V>(k: Decoder<K>, v: Decoder<V>) {
        let self = this;
        return function Table(): Map<K, V> {
            let columnCount = Number(self.read_varint());
            let length = Number(self.read_varint());

            assert(columnCount == 2, RangeError, () => `invalid column count: ${columnCount}`);

            let [field, ty] = self.read_field_id_and_ty();
            if (field == 0) {
                expectedTy(k, ty);
                let keys = Array.from({ length }, () => k.call(self));

                let [field2, ty2] = self.read_field_id_and_ty();
                assert(field2 == 1, TypeError, () => `invalid column (value) id: expected \`1\`, found ${field2}`);
                expectedTy(v, ty2);

                let map = new Map();
                for (let key of keys) map.set(key, v.call(self))
                return map;
            }
            if (field == 1) {
                expectedTy(v, ty);
                let values = Array.from({ length }, () => v.call(self));

                let [field2, ty2] = self.read_field_id_and_ty();
                assert(field2 === 0, TypeError, () => `invalid column (key) id: expected \`0\`, found ${field2}`);
                expectedTy(k, ty2);

                let map = new Map();
                for (let val of values) map.set(k.call(self), val);
                return map;
            }

            throw new TypeError(`invalid column id: expected \`0\` or \`1\`, found ${field}`);
        }
    }
}

function ListType<T, N>(de: Decoder<N>, from: (_: { length: number }, map: () => N) => T) {
    return function List(this: Decode): T {
        let [length, ty] = this.read_len_and_ty();
        expectedTy(de, ty);
        return from({ length }, () => de.call(this));
    }
}

// ================================================================================

function next_field_id_and_ty(de: Deserialize): undefined | [number, DataType] {
    let [id, ty] = de.read_field_id_and_ty();
    if (ty == DataType.StructEnd) {
        assert(id == 0, TypeError, () => `invalid struct end id: ${id}, expected \`0\``);
        return undefined;
    }
    return [id, ty] as const
}

type Schema = readonly [
    name: string,
    id: number,
    decoder: Decoder<unknown>,
    required: boolean
];

// `T[number]` is same as `Array<T1, T2, ..> => union T1 | T2 | ..`
// where E = T1 | T2 | ...
// where E = Schema
//
// ## `E[3] extends true`
// is required == true
//   ? use name (E[0]) as key
//   : ignore
//
// where value is ReturnType of decoder (E[2]) function.
type Transform<T extends readonly Schema[]> =
    // required fields
    { [E in T[number]as E[3] extends true ? E[0] : never]: ReturnType<E[2]>; }
    // optional fields
    & { [E in T[number]as E[3] extends false ? E[0] : never]?: ReturnType<E[2]>; }


export function StructDecoder<const T extends readonly Schema[]>(self: Decode, schemas: T) {
    let obj = {} as Transform<T>;

    let header: [number, DataType] | undefined;
    while (header = next_field_id_and_ty(self)) {
        let [field_id, field_ty] = header;
        // match 
        for (let [name, id, decoder] of schemas) {
            if (id == field_id) {
                if (decoder.name == "Bool") {
                    (obj as any)[name] = DataType.asBool(field_ty);
                } else {
                    expectedTy(decoder, field_ty);
                    (obj as any)[name] = decoder.call(self);
                }
                break;
            }
        }
    }

    // check required fields
    for (let [name, id, de, isRequired] of schemas) {
        if (isRequired && !(name in obj)) {
            throw new Error(`missing required field: ${name} as ${id}; type: ${de.name}`);
        }
    }

    return obj;
}

export function OutputDecoder<T>(self: Decode, de: Decoder<T>, required: true): T;
export function OutputDecoder<T>(self: Decode, de: Decoder<T>, required: false): T | undefined;
export function OutputDecoder<T>(self: Decode, decoder: Decoder<T>, required: boolean) {
    let header = next_field_id_and_ty(self);
    let val: T | undefined;
    if (header) {
        let [field_id, field_ty] = header;
        if (field_id == 0) {
            if (decoder.name == "Bool") {
                //@ts-ignore
                val = DataType.asBool(field_ty);
            } else {
                expectedTy(decoder, field_ty);
                val = decoder.call(self)
            }
        }
    }
    if (required && val !== undefined) {
        throw new Error(`output required, type: ${decoder.name}`);
    }
    return val;
}

// ================================================================================

function view(data: Uint8Array) {
    return new DataView(
        data.buffer,
        data.byteOffset,
        data.byteLength
    )
}

function expectedTy<T>(f: Decoder<T>, ty: DataType) {
    expected(DataType.fromStr(f.name), ty);
}
