import { Buffer } from "../utils/buffer.ts";
import { FrameHeader, LenBE } from "./frame.ts";
import { Trailer } from "./trailer.ts";

export function encodeAsLastFrame(msg: Uint8Array) {
    let len = new LenBE(msg.length);
    let frame = new Buffer();
    frame.appendMany(
        [FrameHeader.new({ lenSize: len.size }).encode()],
        len.asBytes(),
        msg,
        Trailer.OK_ENCODED
    );
    return frame.data();
}
