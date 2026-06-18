import { Encode } from "../lipi/encoder.ts";
import { Status } from "../status.ts";
import { FrameHeader, LenBE } from "./frame.ts";

export function encodeFrame(f: (_: Encode) => void) {
    return encode(f)
}

export function encodeLastFrame(f: (_: Encode) => void) {
    return encode(f, Status.Ok)
}

export function encode(f: (_: Encode) => void, status?: Status) {
    let e = new Encode();
    f(e);

    let len = new LenBE(e.len);
    e.pushFront([FrameHeader.new({ status, lenSize: len.size }).encode()]);
    e.pushFront(len.asBytes());

    return e.data();
}
