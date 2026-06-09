import { Encode } from "../lipi/encoder.ts";
import { encodeTrailer } from "../setu/frame.writer.ts";
import { MPSC } from "../utils/mpsc.ts";

export class Input {
    channel = new MPSC<Uint8Array>();
    controller = new AbortController();

    sendAndClose(f: (this: Encode) => void) {
        let e = new Encode();
        f.call(e);
        this.channel.send(encodeTrailer(e.data()));
        this.channel.close()
    }

    reset(reason?: any) {
        this.controller.abort(reason);
        this.channel.close();
    }
}
