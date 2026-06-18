import { Status } from "../status.ts";
import { Buffer } from "../utils/buffer.ts";
import { FrameHeader, LenBE } from "./frame.ts";

export function encode_frame(msg: Uint8Array, status?: Status | undefined) {
    let len = new LenBE(msg.length);
    let frame = new Buffer();
    frame.appendMany(
        [FrameHeader.new({ status, lenSize: len.size }).encode()],
        len.asBytes(),
        msg,
    );
    return frame.data();
}
