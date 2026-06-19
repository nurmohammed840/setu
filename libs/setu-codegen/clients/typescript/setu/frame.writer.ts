import { Encode } from "../lipi/encoder.ts";
import { Status } from "../status.ts";
import { Buffer } from "../utils/buffer.ts";
import { assert } from "../utils/common.ts";
import { Trailer } from "./trailer.ts";
import { FrameHeader, LenBE } from "./frame.ts";

export function encodeFrame(f: (_: Encode) => void) {
    return encode(f)
}

export function encodeLastFrame(f: (_: Encode) => void) {
    return encode(f, Status.Ok)
}

export function encodeErrorFrame(status: Status, reason?: string) {
    assert(status != Status.Ok);
    return encode(e => Trailer.encode.call(e, Trailer.error(reason)), status);
}

export function encode(encoder: (_: Encode) => void, status?: Status) {
    let e = new Encode();
    encoder(e);
    e.pushFront(encodeHeader(e.len, status));
    return e.data();
}

function encodeHeader(len: number, status?: Status) {
    let len_be = new LenBE(len);

    let buf = new Buffer();
    buf.writeByte(FrameHeader.new({ status, lenSize: len_be.size }).encode());
    buf.push(len_be.asBytes());
    return buf.data()
}