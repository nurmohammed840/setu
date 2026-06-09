import { Status } from "../status.ts";
import { Buffer } from "../utils/buffer.ts";
import { FrameHeader, LenBE } from "./frame.ts";

export function encodeAsLastFrame(msg: Uint8Array) {
    let len = new LenBE(msg.length);
    let frame = new Buffer();
    frame.appendMany(
        [FrameHeader.new({ status: Status.Ok, lenSize: len.size }).encode()],
        len.asBytes(),
        msg,
    );
    return frame.data();
}
