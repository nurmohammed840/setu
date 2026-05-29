import { Encode, StructEncoder } from "./lipi/encoder.ts";
import { encodeAsLastFrame } from "./setu/frame.writer.ts";
import { MPSC } from "./utils/mpsc.ts";

export class Input {
    channel = new MPSC<Uint8Array>();

    sendAndClose(f: (s: StructEncoder) => void) {
        let e = new Encode();

        let s = new StructEncoder(e);
        f(s);
        s.end();

        this.channel.send(encodeAsLastFrame(e.data()));
        this.channel.close()
    }
}

