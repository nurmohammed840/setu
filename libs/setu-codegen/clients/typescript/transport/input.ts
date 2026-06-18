// import { Encode } from "../lipi/encoder.ts";
// import { encode_frame } from "../setu/frame.writer.ts";
import { MPSC } from "../utils/mpsc.ts";
export class Stream<T, R> {
    channel = new MPSC<Uint8Array>();
}
