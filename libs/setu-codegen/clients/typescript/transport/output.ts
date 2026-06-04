import { Decode, Decoder } from "../lipi/decoder.ts";
import { FrameDecoder } from "../setu/frame.ts";
import { Status } from "../status.ts";
import { Bytes } from "../utils/bytes.ts";
import { assert } from "../utils/common.ts";
import { Stream } from "../utils/stream.ts";
import { Input } from "./input.ts";

export interface Output<T> extends Promise<T> {
    cancle(reason?: any): void;
}

export function Output<T>(input: Input, futRes: Promise<ReadableStream<Uint8Array>>, f: Decoder<T>) {
    let fut = Promise.withResolvers<T>();
    let stream: Stream | undefined;
    let canceled: { reason?: any } | undefined;

    let output: Output<T> = Object.assign(fut.promise, {
        cancle(reason?: any) {
            input.reset(reason);
            stream?.reader.cancel(reason);
            canceled = { reason };
        }
    });

    (async () => {
        try {
            let res = await futRes;

            if (canceled) {
                res.cancel(canceled.reason);
                return fut.reject(canceled.reason);
            }

            stream = new Stream(res.getReader());
            let reader = new FrameDecoder(stream);
            let dataFrame = await reader.parseFrame();
            let trailer = await reader.parseFrame();

            assert(dataFrame.data.type == "message");
            assert(trailer.data.type == "trailer", Error, `expected trailer`);
            assert(trailer.data.status == Status.Ok, Error, `trailer status: ${trailer.data.status}`);

            let de = new Decode(new Bytes(dataFrame.data.bytes));
            fut.resolve(f.call(de));
        } catch (error) {
            fut.reject(error)
            // output.cancle()
        }
    })();

    return output;
}
