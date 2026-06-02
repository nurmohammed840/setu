import { Encode } from "./lipi/encoder.ts";
import { encodeAsLastFrame } from "./setu/frame.writer.ts";
import { MPSC } from "./utils/mpsc.ts";

export class Input {
    channel = new MPSC<Uint8Array>();

    sendAndClose(f: (this: Encode) => void) {
        let e = new Encode();
        f.call(e);
        this.channel.send(encodeAsLastFrame(e.data()));
        this.channel.close()
    }
}

