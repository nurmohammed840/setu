function expected(data, err = "expected value") {
    assert(data != undefined, TypeError, err);
    return data;
}
function assert(expr, err = Error, msg = "") {
    if (expr) return;
    const e = new err(typeof msg === "string" ? msg : msg());
    Error.captureStackTrace(e, assert);
    throw e;
}
function checkOverflow(num, min, max) {
    if (num < min || num > max) {
        throw new RangeError(`expected ${min}..=${max}, got: ${num}`);
    }
}
function encodeVarInt(num) {
    num = BigInt(num);
    assert(num >= 0n, RangeError, ()=>`expected unsigned number: found ${num}`);
    let buf = [];
    while(num > 0b111_1111){
        buf.push(Number(num & 0xFFn | 0b1000_0000n));
        num >>= 7n;
    }
    buf.push(Number(num));
    return new Uint8Array(buf);
}
function decodeVarInt(bytes) {
    let result = 0n;
    let shift = 0n;
    while(true){
        let __byte = BigInt(bytes.nextByte());
        if (shift == 63n && __byte >= 2) throw new Error("invalid variable-length integer");
        if ((__byte & 0b1000_0000n) == 0n) return result | __byte << shift;
        result |= (__byte & 0b111_1111n) << shift;
        shift += 7n;
    }
}
function zigzagEncode(num) {
    return num << 1n ^ num >> 63n;
}
function zigzagDecode(num) {
    return num >> 1n ^ -(num & 1n);
}
class Buffer {
    #len = 0;
    #data = [];
    append(buf) {
        this.#len += buf.length;
        this.#data.push(buf);
    }
    appendMany(...bufs) {
        for (let buf of bufs)this.#len += buf.length;
        this.#data.push(...bufs);
    }
    writeByte(__byte) {
        checkOverflow(__byte, 0, 255);
        this.#len += 1;
        this.#data.push([
            __byte
        ]);
    }
    get len() {
        return this.#len;
    }
    data() {
        let b = new Uint8Array(this.#len);
        let i = 0;
        for (let buf of this.#data){
            b.set(buf, i);
            i += buf.length;
        }
        return b;
    }
}
class BitVec {
    bytes;
    constructor(lenOrBytes){
        if (typeof lenOrBytes === "number") {
            this.bytes = new Uint8Array(boolPackedLen(lenOrBytes));
        } else {
            this.bytes = lenOrBytes;
        }
    }
    asBytes() {
        return this.bytes;
    }
    capacity() {
        return this.bytes.length * 8;
    }
    isEmpty() {
        return this.bytes.every((slot)=>slot == 0);
    }
    has(index) {
        return this.get(index) ?? false;
    }
    get(index) {
        const slotIdx = Math.floor(index / 8);
        const mask = 1 << index % 8;
        const slot = this.bytes[slotIdx];
        if (slot === undefined) {
            return undefined;
        }
        return (slot & mask) !== 0;
    }
    clear() {
        this.bytes.fill(0);
    }
    set(index) {
        const slotIdx = Math.floor(index / 8);
        const mask = 1 << index % 8;
        assert(slotIdx < this.bytes.length, RangeError, ()=>`Out of bounds slot index: ${slotIdx}`);
        const oldValue = (this.bytes[slotIdx] & mask) !== 0;
        this.bytes[slotIdx] |= mask;
        return oldValue;
    }
    remove(index) {
        const slotIdx = Math.floor(index / 8);
        const mask = 1 << index % 8;
        if (slotIdx >= this.bytes.length) {
            return undefined;
        }
        const oldValue = (this.bytes[slotIdx] & mask) !== 0;
        this.bytes[slotIdx] &= ~mask;
        return oldValue;
    }
}
function boolPackedLen(len) {
    assert(len >= 0, RangeError, ()=>`length ${len} cannot be negative`);
    return Math.floor((len + 7) / 8);
}
function bitvec(len) {
    return new BitVec(len);
}
function bitvecFrom(bools) {
    const bv = bitvec(bools.length);
    for(let i = 0; i < bools.length; i++){
        if (bools[i]) {
            bv.set(i);
        }
    }
    return bv;
}
function takeBytes(N, buf) {
    assert(N <= buf.length, RangeError, ()=>`takeBytes(${N}) exceeds buffer length ${buf.length}`);
    return [
        buf.subarray(0, N),
        buf.subarray(N)
    ];
}
class Bytes {
    data;
    static empty() {
        return new Bytes(new Uint8Array());
    }
    constructor(data){
        this.data = data;
    }
    get length() {
        return this.remaining().length;
    }
    isEmpty() {
        return this.length == 0;
    }
    nextByte() {
        let [[__byte], ptr] = takeBytes(1, this.data);
        this.data = ptr;
        return __byte;
    }
    take(len) {
        let [bytes, ptr] = takeBytes(len, this.data);
        this.data = ptr;
        return bytes;
    }
    remaining() {
        return this.data;
    }
}
function isLittleEndian() {
    const buf = new ArrayBuffer(4);
    new Uint32Array(buf)[0] = 0x11_22_33_44;
    return new Uint8Array(buf)[0] == 0x44;
}
const IS_LITTLE_ENDIAN = isLittleEndian();
var DataType;
(function(DataType) {
    DataType[DataType["False"] = 0] = "False";
    DataType[DataType["True"] = 1] = "True";
    DataType[DataType["U8"] = 2] = "U8";
    DataType[DataType["I8"] = 3] = "I8";
    DataType[DataType["F32"] = 4] = "F32";
    DataType[DataType["F64"] = 5] = "F64";
    DataType[DataType["UInt"] = 6] = "UInt";
    DataType[DataType["Int"] = 7] = "Int";
    DataType[DataType["Str"] = 8] = "Str";
    DataType[DataType["Struct"] = 9] = "Struct";
    DataType[DataType["StructEnd"] = 10] = "StructEnd";
    DataType[DataType["Union"] = 11] = "Union";
    DataType[DataType["List"] = 12] = "List";
    DataType[DataType["Table"] = 13] = "Table";
    DataType[DataType["UnknownI"] = 14] = "UnknownI";
    DataType[DataType["UnknownII"] = 15] = "UnknownII";
})(DataType || (DataType = {}));
(function(DataType) {
    function fromBool(bool) {
        return +bool;
    }
    DataType.fromBool = fromBool;
    function fromStr(str) {
        let ty = DataType[str];
        assert(ty !== undefined, TypeError, ()=>`invalid type: ${str}`);
        return ty;
    }
    DataType.fromStr = fromStr;
})(DataType || (DataType = {}));
const utf8Encoder = new TextEncoder();
class Writer extends Buffer {
    writeUint(num) {
        this.append(encodeVarInt(num));
    }
    writeBytes(bytes) {
        this.writeUint(bytes.length);
        this.append(bytes);
    }
    write_field_id_and_ty(num, ty) {
        assert(Number.isInteger(num) && num >= 0, RangeError, ()=>`expected non-negative integer, got: ${num}`);
        if (num < 15) return this.writeByte(num << 4 | ty);
        this.writeByte(0b1111 << 4 | ty);
        this.writeUint(num - 15);
    }
    write_len_and_ty(num, ty) {
        assert(num <= 0xFF_FF_FF_F);
        this.write_field_id_and_ty(num, ty);
    }
}
class Encode extends Writer {
    U8(num) {
        this.writeByte(num);
    }
    I8(num) {
        checkOverflow(num, -128, 127);
        const buf = new ArrayBuffer(1);
        new DataView(buf).setInt8(0, num);
        this.append(new Uint8Array(buf));
    }
    F32(num) {
        const buf = new ArrayBuffer(4);
        new DataView(buf).setFloat32(0, num, true);
        this.append(new Uint8Array(buf));
    }
    F64(num) {
        const buf = new ArrayBuffer(8);
        new DataView(buf).setFloat64(0, num, true);
        this.append(new Uint8Array(buf));
    }
    UInt(num) {
        this.writeUint(num);
    }
    Int(num) {
        this.writeUint(zigzagEncode(BigInt(num)));
    }
    Str(text) {
        this.writeBytes(utf8Encoder.encode(text));
    }
    List(f) {
        let self = this;
        return function List(vals) {
            self.write_len_and_ty(vals.length, DataType.fromStr(f.name));
            for (let v of vals)f.call(self, v);
        };
    }
    ListU8 = function List(v) {
        this.write_len_and_ty(v.length, DataType.U8);
        this.append(v);
    };
    ListI8 = function List(v) {
        this.write_len_and_ty(v.length, DataType.I8);
        this.append(RawBytes(v));
    };
    ListF32 = function List(v) {
        this.write_len_and_ty(v.length, DataType.F32);
        if (IS_LITTLE_ENDIAN) return this.append(RawBytes(v));
        for (let n of v)this.F32(n);
    };
    ListF64 = function List(v) {
        this.write_len_and_ty(v.length, DataType.F64);
        if (IS_LITTLE_ENDIAN) return this.append(RawBytes(v));
        for (let n of v)this.F64(n);
    };
    ListBool = function List(bools) {
        this.write_len_and_ty(bools.length, DataType.True);
        this.append(bitvecFrom(bools).asBytes());
    };
    Table(k, v) {
        let self = this;
        return function Table(map) {
            self.writeUint(2);
            self.writeUint(map.size);
            self.write_field_id_and_ty(0, DataType.fromStr(k.name));
            for (let key of map.keys())k.call(self, key);
            self.write_field_id_and_ty(1, DataType.fromStr(v.name));
            for (let val of map.values())v.call(self, val);
        };
    }
}
class StructEncoder {
    e;
    constructor(e){
        this.e = e;
    }
    Field(f) {
        return (id, v)=>{
            this.e.write_field_id_and_ty(id, DataType.fromStr(f.name));
            f.call(this.e, v);
        };
    }
    Option(f) {
        return (id, val)=>{
            if (val === undefined) return;
            f.call(this, id, val);
        };
    }
    Bool(id, bool) {
        this.e.write_field_id_and_ty(id, DataType.fromBool(bool));
    }
    get U8() {
        return this.Field(this.e.U8);
    }
    get I8() {
        return this.Field(this.e.I8);
    }
    get F32() {
        return this.Field(this.e.F32);
    }
    get F64() {
        return this.Field(this.e.F64);
    }
    get UInt() {
        return this.Field(this.e.UInt);
    }
    get Int() {
        return this.Field(this.e.Int);
    }
    get Str() {
        return this.Field(this.e.Str);
    }
    List(f) {
        return this.Field(this.e.List(f));
    }
    get ListU8() {
        return this.Field(this.e.ListU8);
    }
    get ListI8() {
        return this.Field(this.e.ListI8);
    }
    get ListF32() {
        return this.Field(this.e.ListF32);
    }
    get ListF64() {
        return this.Field(this.e.ListF64);
    }
    get ListUint() {
        return this.List(this.e.UInt);
    }
    get ListInt() {
        return this.List(this.e.Int);
    }
    get ListStr() {
        return this.List(this.e.Str);
    }
    get ListBool() {
        return this.Field(this.e.ListBool);
    }
    Table(k, v) {
        return this.Field(this.e.Table(k, v));
    }
    end() {
        this.e.writeByte(DataType.StructEnd);
    }
}
function RawBytes(v) {
    return new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
}
const mod = {
    DataType: DataType,
    Writer,
    Encode,
    StructEncoder,
    encodeVarInt,
    decodeVarInt,
    zigzagEncode,
    zigzagDecode
};
var Status;
(function(Status) {
    Status[Status["Ok"] = 0] = "Ok";
    Status[Status["Cancelled"] = 1] = "Cancelled";
    Status[Status["Unknown"] = 2] = "Unknown";
    Status[Status["DeadlineExceeded"] = 4] = "DeadlineExceeded";
    Status[Status["PermissionDenied"] = 7] = "PermissionDenied";
    Status[Status["ResourceExhausted"] = 8] = "ResourceExhausted";
    Status[Status["Unimplemented"] = 12] = "Unimplemented";
    Status[Status["Internal"] = 13] = "Internal";
    Status[Status["Unavailable"] = 14] = "Unavailable";
    Status[Status["InvalidArgument"] = 3] = "InvalidArgument";
    Status[Status["NotFound"] = 5] = "NotFound";
    Status[Status["AlreadyExists"] = 6] = "AlreadyExists";
    Status[Status["FailedPrecondition"] = 9] = "FailedPrecondition";
    Status[Status["Aborted"] = 10] = "Aborted";
    Status[Status["OutOfRange"] = 11] = "OutOfRange";
    Status[Status["DataLoss"] = 15] = "DataLoss";
})(Status || (Status = {}));
(function(Status) {
    function from(code) {
        return code & 0b1111;
    }
    Status.from = from;
    function toString(status) {
        return Status[status];
    }
    Status.toString = toString;
})(Status || (Status = {}));
export { Status as Status };
class EndOfData extends Error {
}
class ProtocolError extends Error {
}
var _computedKey, _computedKey1;
_computedKey = Symbol.dispose;
class Stream {
    reader;
    eos;
    constructor(reader){
        this.reader = reader;
        this.eos = false;
    }
    [_computedKey]() {
        this.reader.cancel();
    }
    async read() {
        assert(!this.eos, EndOfData, "read after eos");
        const { done, value } = await this.reader.read();
        if (done) {
            this.eos = true;
            return;
        }
        return value;
    }
    async toBytes() {
        let chunk;
        let buf = new Buffer();
        while(chunk = await this.read())buf.append(chunk);
        return buf.data();
    }
}
_computedKey1 = Symbol.dispose;
class StreamReader {
    stream;
    data;
    constructor(stream){
        this.stream = stream;
        this.data = Bytes.empty();
    }
    [_computedKey1]() {
        this.stream[Symbol.dispose]();
    }
    async readBytes(len) {
        if (len == 0) {
            return new Uint8Array();
        }
        let data = await this.read();
        if (len <= data.length) {
            return data.take(len);
        }
        let buf = new Buffer();
        while(buf.len < len){
            let data = await this.read();
            let remaining = len - buf.len;
            let takeN = Math.min(remaining, data.length);
            buf.append(data.take(takeN));
        }
        return buf.data();
    }
    async readByte() {
        let data = await this.read();
        return data.nextByte();
    }
    async read() {
        while(this.data.isEmpty()){
            let bytes = expected(await this.stream.read(), "unexpected end of message");
            this.data = new Bytes(bytes);
        }
        return this.data;
    }
}
class MaybeCompressed {
    isCompressed;
    data;
    constructor(isCompressed, data){
        this.isCompressed = isCompressed;
        this.data = data;
    }
}
class FrameHeader {
    isCompressed;
    isTrailer;
    lenSize;
    code;
    constructor(isCompressed, isTrailer, lenSize, code){
        this.isCompressed = isCompressed;
        this.isTrailer = isTrailer;
        this.lenSize = lenSize;
        this.code = code;
    }
    static new = ({ lenSize, trailer, isCompressed })=>new FrameHeader(!!isCompressed, trailer != undefined, lenSize - 1, trailer ?? 0);
    static parse = (__byte)=>new FrameHeader((__byte & 0b1) === 0b1, (__byte & 0b10) === 0b10, (__byte >> 2 & 0b11) + 1, __byte >> 4);
    encode() {
        return this.code << 4 | this.lenSize << 2 | +this.isTrailer << 1 | +this.isCompressed;
    }
}
class LenBE {
    #buf = new ArrayBuffer(4);
    size;
    constructor(len){
        new DataView(this.#buf).setUint32(0, len, false);
        if (len <= 0xFF) this.size = 1;
        else if (len <= 0xFF_FF) this.size = 2;
        else if (len <= 0xFF_FF_FF) this.size = 3;
        else {
            assert(len <= 0xFF_FF_FF_FF, RangeError, ()=>`len: ${len} must fit in u32`);
            this.size = 4;
        }
    }
    asBytes() {
        return new Uint8Array(this.#buf, 4 - this.size);
    }
}
class FrameDecoder extends StreamReader {
    async parseFrame() {
        let header = FrameHeader.parse(await this.readByte());
        let len = await this.parseLenBigEndian(header.lenSize);
        let bytes = await this.readBytes(len);
        return new MaybeCompressed(header.isCompressed, header.isTrailer ? {
            type: "trailer",
            status: Status.from(header.code),
            bytes
        } : {
            type: "message",
            bytes
        });
    }
    async parseLenBigEndian(size) {
        let len = 0;
        for(let i = 0; i < size; i++){
            len = len << 8 | await this.readByte();
        }
        return 0;
    }
}
class Trailer {
    static OK_ENCODED = [
        FrameHeader.new({
            lenSize: 1,
            trailer: Status.Ok
        }).encode(),
        0
    ];
}
function encodeAsLastFrame(msg) {
    let len = new LenBE(msg.length);
    let frame = new Buffer();
    frame.appendMany([
        FrameHeader.new({
            lenSize: len.size
        }).encode()
    ], len.asBytes(), msg, Trailer.OK_ENCODED);
    return frame.data();
}
const mod1 = {
    MaybeCompressed,
    FrameHeader,
    LenBE,
    FrameDecoder,
    encodeAsLastFrame,
    Trailer
};
var _computedKey2;
_computedKey2 = Symbol.dispose;
class MPSC {
    #controller;
    #closed = false;
    stream = new ReadableStream({
        start: (controller)=>{
            this.#controller = controller;
        },
        cancel: ()=>{
            this.#closed = true;
        }
    });
    [_computedKey2]() {
        this.close();
    }
    send(val) {
        assert(!this.#closed, Error, "Channel is closed");
        this.#controller.enqueue(val);
    }
    close() {
        this.#closed = true;
        this.#controller.close();
    }
    get isClosed() {
        return this.#closed;
    }
}
class Input {
    channel = new MPSC();
    sendAndClose(f) {
        let e = new Encode();
        let s = new StructEncoder(e);
        f(s);
        s.end();
        this.channel.send(encodeAsLastFrame(e.data()));
        this.channel.close();
    }
}
var TimeoutUnit;
(function(TimeoutUnit) {
    TimeoutUnit["Hour"] = "H";
    TimeoutUnit["Minute"] = "M";
    TimeoutUnit["Second"] = "S";
    TimeoutUnit["Millisecond"] = "m";
})(TimeoutUnit || (TimeoutUnit = {}));
class Timeout {
    unit;
    value;
    constructor(unit, value){
        this.unit = unit;
        this.value = value;
    }
    static hour(hours) {
        return new Timeout(TimeoutUnit.Hour, hours);
    }
    static minute(mins) {
        return new Timeout(TimeoutUnit.Minute, mins);
    }
    static second(secs) {
        return new Timeout(TimeoutUnit.Second, secs);
    }
    static millisecond(ms) {
        return new Timeout(TimeoutUnit.Millisecond, ms);
    }
    toString() {
        return `${this.value}${this.unit}`;
    }
    duration() {
        switch(this.unit){
            case TimeoutUnit.Hour:
                return this.value * 60 * 60 * 1000;
            case TimeoutUnit.Minute:
                return this.value * 60 * 1000;
            case TimeoutUnit.Second:
                return this.value * 1000;
            case TimeoutUnit.Millisecond:
                return this.value;
        }
    }
    static fromString(input) {
        assert(input.length >= 2, SyntaxError, "timeout: invalid format");
        const numPart = input.slice(0, -1);
        const unit = input.slice(-1);
        const value = Number.parseInt(numPart, 10);
        assert(Number.isFinite(value), TypeError, "timeout: invalid number");
        switch(unit){
            case "H":
                return Timeout.hour(value);
            case "M":
                return Timeout.minute(value);
            case "S":
                return Timeout.second(value);
            case "m":
                return Timeout.millisecond(value);
            default:
                throw new Error("timeout: unknown unit");
        }
    }
    equals(other) {
        console.log({
            other
        });
        return this.unit === other.unit && this.value === other.value;
    }
}
export { TimeoutUnit as TimeoutUnit };
export { Timeout as Timeout };
class RPC {
    static URL = new URL("/", "https://localhost:443");
    static TIMEOUT = Timeout.minute(2);
    static async call(id, body, timeout = RPC.TIMEOUT, url = RPC.URL) {
        let headers = {
            "content-type": "application/setu",
            "rpc-id": id.toString()
        };
        if (timeout) {
            headers["rpc-timeout"] = timeout.toString();
        }
        let res = await fetch(url, {
            method: "POST",
            headers,
            body
        });
        if (!res.ok) {
            throw new Error(`${res.statusText}: ${await res.text()}`);
        }
        let contentType = res.headers.get("content-type");
        assert(contentType == "application/setu", ProtocolError, ()=>`unexpected content-type: ${contentType ?? "none"}`);
        assert(res.body, ProtocolError, "No response body");
        return res.body;
    }
}
function rpc(id, { timeout, url }) {
    let input = new Input();
    let output = RPC.call(id, input.channel.stream, timeout, url);
    return [
        input,
        output
    ];
}
export { RPC as RPC };
export { rpc as rpc };
function Obj(f) {
    return function Struct(args) {
        let s = new StructEncoder(this);
        f(s, args);
        s.end();
    };
}
export { Obj as Obj };
export { mod as lipi };
export { mod1 as setu };
