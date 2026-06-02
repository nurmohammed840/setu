declare module "utils/common" {
    type ErrorClass = new (message?: string) => Error;
    type ErrorMessage = (() => string) | string;
    export function expected<T>(data?: T, err?: string): T;
    export function assert(expr: unknown, err?: ErrorClass, msg?: ErrorMessage): asserts expr;
    export const IS_LITTLE_ENDIAN: boolean;
    export function checkOverflow(num: number, bit: number, signed?: boolean): number;
}
declare module "utils/bytes" {
    export function takeBytes(N: number, buf: Uint8Array): [Uint8Array, Uint8Array];
    export class Bytes {
        private data;
        static empty(): Bytes;
        constructor(data: Uint8Array);
        get length(): number;
        isEmpty(): boolean;
        nextByte(): number;
        take(len: number): Uint8Array;
        remaining(): Uint8Array;
    }
}
declare module "lipi/varint" {
    import { Bytes } from "utils/bytes";
    export function encodeVarInt(num: bigint | number): Uint8Array;
    export function decodeVarInt(bytes: Bytes): bigint;
}
declare module "lipi/type" {
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
        UnknownII = 15
    }
    export namespace DataType {
        function fromBool(bool: boolean): DataType;
        function asBool(ty: DataType): ty is DataType.True;
        function fromStr(str: string): DataType;
        function expected(expected: DataType, found: DataType): void;
    }
}
declare module "lipi/zigzag" {
    export function zigzagEncode(num: bigint): bigint;
    export function zigzagDecode(num: bigint): bigint;
}
declare module "bitset" {
    export interface BitSetRead {
        capacity(): number;
        isEmpty(): boolean;
        has(index: number): boolean;
        get(index: number): boolean | undefined;
    }
    export interface BitSetWrite {
        clear(): void;
        set(index: number): boolean;
        remove(index: number): boolean | undefined;
    }
    export class BitVec implements BitSetRead, BitSetWrite {
        private bytes;
        constructor(lenOrBytes: number | Uint8Array);
        asBytes(): Uint8Array;
        capacity(): number;
        isEmpty(): boolean;
        has(index: number): boolean;
        get(index: number): boolean | undefined;
        clear(): void;
        set(index: number): boolean;
        remove(index: number): boolean | undefined;
    }
    export function boolPackedLen(len: number): number;
    export function bitvec(len: number | Uint8Array): BitVec;
    export function bitvecFrom(bools: boolean[]): BitVec;
    export function bitvecToBools(bitvec: Uint8Array | BitVec, len: number): boolean[];
}
declare module "lipi/decoder" {
    import { Bytes } from "utils/bytes";
    import { DataType } from "lipi/type";
    type Decoder<T> = (this: Decode) => T;
    export class Deserialize {
        buf: Bytes;
        constructor(buf: Bytes);
        read_varint(): bigint;
        read_len(): number;
        read_bytes(): Uint8Array;
        read_field_id_and_ty(): readonly [number, DataType];
        read_len_and_ty(): readonly [number, DataType];
    }
    export class Decode extends Deserialize {
        Bool(): boolean;
        U8(): number;
        I8(): number;
        F32(): number;
        F64(): number;
        U16: (this: Decode) => bigint;
        U32: (this: Decode) => bigint;
        U64: (this: Decode) => bigint;
        Int(): bigint;
        I16: (this: Decode) => number;
        I32: (this: Decode) => bigint;
        I64: (this: Decode) => bigint;
        Str(): string;
        List<T>(f: Decoder<T>): () => Array<T>;
        ListU8(): Uint8Array;
        ListI8(): Int8Array;
        ListF32(): Float32Array;
        ListF64(): Float64Array;
        ListU16(): Uint16Array;
        ListU32(): Uint32Array;
        ListU64(): BigUint64Array;
        ListI16(): Int16Array;
        ListI32(): Int32Array;
        ListI64(): BigInt64Array;
        ListBool(): boolean[];
        Table<K, V>(k: Decoder<K>, v: Decoder<V>): () => Map<K, V>;
    }
    type Schema = readonly [string, number, Decoder<unknown>, boolean];
    type Transform<T extends readonly Schema[]> = {
        [E in T[number] as E[3] extends true ? E[0] : never]: ReturnType<E[2]>;
    } & {
        [E in T[number] as E[3] extends false ? E[0] : never]?: ReturnType<E[2]>;
    };
    export function StructDecoder<T extends Schema[]>(self: Decode, schemas: T): Transform<T>;
}
declare module "utils/buffer" {
    export class Buffer {
        #private;
        append(buf: ArrayLike<number>): void;
        appendMany(...bufs: ArrayLike<number>[]): void;
        writeByte(byte: number): void;
        get len(): number;
        data(): Uint8Array;
    }
}
declare module "lipi/encoder" {
    import { DataType } from "lipi/type";
    import { Buffer } from "utils/buffer";
    type Encoder<T> = (this: Encode, val: T) => void;
    export class Writer extends Buffer {
        writeVarint(num: number | bigint): void;
        writeBytes(bytes: Uint8Array): void;
        write_field_id_and_ty(num: number, ty: DataType): void;
        write_len_and_ty(num: number, ty: DataType): void;
    }
    export class Encode extends Writer {
        U8(num: number): void;
        I8(num: number): void;
        F32(num: number): void;
        F64(num: number): void;
        UInt(num: number | bigint): void;
        Int(num: number | bigint): void;
        Str(text: string): void;
        List<T>(f: Encoder<T>): (vals: Iterable<T> & {
            length: number;
        }) => void;
        ListU8: (this: Encode, v: Uint8Array) => void;
        ListI8: (this: Encode, v: Int8Array) => void;
        ListF32: (this: Encode, v: Float32Array) => void;
        ListF64: (this: Encode, v: Float64Array) => void;
        ListBool: (this: Encode, bools: Array<boolean>) => void;
        Table<K, V>(k: Encoder<K>, v: Encoder<V>): (map: Map<K, V>) => void;
    }
    export class StructEncoder {
        e: Encode;
        constructor(e: Encode);
        Field<T>(f: Encoder<T>): (id: number, v: T) => void;
        Option<T>(f: (this: this, id: number, value: T) => void): (id: number, val?: T) => void;
        Bool(id: number, bool: boolean): void;
        get U8(): (id: number, v: number) => void;
        get I8(): (id: number, v: number) => void;
        get F32(): (id: number, v: number) => void;
        get F64(): (id: number, v: number) => void;
        get UInt(): (id: number, v: number | bigint) => void;
        get Int(): (id: number, v: number | bigint) => void;
        get Str(): (id: number, v: string) => void;
        List<T>(f: Encoder<T>): (id: number, v: Iterable<T> & {
            length: number;
        }) => void;
        get ListU8(): (id: number, v: Uint8Array) => void;
        get ListI8(): (id: number, v: Int8Array) => void;
        get ListF32(): (id: number, v: Float32Array) => void;
        get ListF64(): (id: number, v: Float64Array) => void;
        get ListUint(): (id: number, v: Iterable<number | bigint> & {
            length: number;
        }) => void;
        get ListInt(): (id: number, v: Iterable<number | bigint> & {
            length: number;
        }) => void;
        get ListStr(): (id: number, v: Iterable<string> & {
            length: number;
        }) => void;
        get ListBool(): (id: number, v: boolean[]) => void;
        Table<K, V>(k: Encoder<K>, v: Encoder<V>): (id: number, v: Map<K, V>) => void;
        end(): void;
    }
}
declare module "lipi/mod" {
    export * from "lipi/decoder";
    export * from "lipi/encoder";
    export * from "lipi/varint";
    export * from "lipi/zigzag";
}
declare module "status" {
    export enum Status {
        Ok = 0,
        Cancelled = 1,
        Unknown = 2,
        DeadlineExceeded = 4,
        PermissionDenied = 7,
        ResourceExhausted = 8,
        Unimplemented = 12,
        Internal = 13,
        Unavailable = 14,
        InvalidArgument = 3,
        NotFound = 5,
        AlreadyExists = 6,
        FailedPrecondition = 9,
        Aborted = 10,
        OutOfRange = 11,
        DataLoss = 15
    }
    export namespace Status {
        function from(code: number): Status;
        function toString(status: Status): string;
    }
}
declare module "errors" {
    export class EndOfData extends Error {
    }
    export class ProtocolError extends Error {
    }
}
declare module "utils/stream" {
    import { Bytes } from "utils/bytes";
    export class Stream {
        private reader;
        eos: boolean;
        constructor(reader: ReadableStreamDefaultReader<Uint8Array>);
        [Symbol.dispose](): void;
        read(): Promise<Uint8Array | undefined>;
        toBytes(): Promise<Uint8Array>;
    }
    export class StreamReader {
        stream: Stream;
        data: Bytes;
        constructor(stream: Stream);
        [Symbol.dispose](): void;
        readBytes(len: number): Promise<Uint8Array>;
        readByte(): Promise<number>;
        read(): Promise<Bytes>;
    }
}
declare module "setu/frame" {
    import { Status } from "status";
    import { StreamReader } from "utils/stream";
    export class MaybeCompressed<T> {
        private isCompressed;
        private data;
        constructor(isCompressed: boolean, data: T);
    }
    export type Frame = MessageFrame | TrailerFrame;
    export interface MessageFrame {
        type: "message";
        bytes: Uint8Array;
    }
    export interface TrailerFrame {
        type: "trailer";
        status: Status;
        bytes: Uint8Array;
    }
    interface FrameHeaderArgs {
        lenSize: number;
        isCompressed?: boolean;
        trailer?: Status;
    }
    export class FrameHeader {
        isCompressed: boolean;
        isTrailer: boolean;
        lenSize: number;
        code: number;
        constructor(isCompressed: boolean, isTrailer: boolean, lenSize: number, code: number);
        static new: ({ lenSize, trailer, isCompressed }: FrameHeaderArgs) => FrameHeader;
        static parse: (byte: number) => FrameHeader;
        encode(): number;
    }
    export class LenBE {
        #private;
        size: number;
        constructor(len: number);
        asBytes(): Uint8Array;
    }
    export class FrameDecoder extends StreamReader {
        parseFrame(): Promise<MaybeCompressed<Frame>>;
        parseLenBigEndian(size: number): Promise<number>;
    }
}
declare module "setu/trailer" {
    export class Trailer {
        static OK_ENCODED: number[];
    }
}
declare module "setu/frame.writer" {
    export function encodeAsLastFrame(msg: Uint8Array): Uint8Array;
}
declare module "setu/mod" {
    export * from "setu/frame";
    export * from "setu/frame.writer";
    export * from "setu/trailer";
}
declare module "utils/mpsc" {
    export class MPSC<T> {
        #private;
        readonly stream: ReadableStream<T>;
        [Symbol.dispose](): void;
        send(val: T): void;
        close(): void;
        get isClosed(): boolean;
    }
}
declare module "input" {
    import { StructEncoder } from "lipi/encoder";
    import { MPSC } from "utils/mpsc";
    export class Input {
        channel: MPSC<Uint8Array>;
        sendAndClose(f: (s: StructEncoder) => void): void;
    }
}
declare module "timeout" {
    export enum TimeoutUnit {
        Hour = "H",
        Minute = "M",
        Second = "S",
        Millisecond = "m"
    }
    export class Timeout {
        readonly unit: TimeoutUnit;
        readonly value: number;
        private constructor();
        static hour(hours: number): Timeout;
        static minute(mins: number): Timeout;
        static second(secs: number): Timeout;
        static millisecond(ms: number): Timeout;
        toString(): string;
        duration(): number;
        static fromString(input: string): Timeout;
        equals(other: Timeout): boolean;
    }
}
declare module "http.transport" {
    import { Input } from "input";
    import { Timeout } from "timeout";
    export class RPC {
        static URL: URL;
        static TIMEOUT: Timeout;
        static call(id: number, body: BodyInit, timeout?: Timeout | null, url?: URL): Promise<ReadableStream<Uint8Array>>;
    }
    export interface Context {
        url?: URL;
        timeout?: Timeout | null;
    }
    export function rpc(id: number, { timeout, url }: Context): readonly [Input, Promise<ReadableStream<Uint8Array>>];
}
declare module "helper" {
    import { Encode, StructEncoder } from "lipi/mod";
    export function Obj<T>(f: (s: StructEncoder, args: T) => void): (this: Encode, args: T) => void;
}
declare module "mod" {
    export * as lipi from "lipi/mod";
    export * as setu from "setu/mod";
    export * from "http.transport";
    export * from "status";
    export * from "timeout";
    export * from "helper";
}
