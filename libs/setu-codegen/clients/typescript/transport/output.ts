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

export function Output<T>(
    controller: AbortController,
    readableStream: Promise<ReadableStream<Uint8Array>>,
    decoder: Decoder<T>
) {
    let fut = Promise.withResolvers<T>();
    let stream: Stream | undefined;
    let canceled: { reason?: any } | undefined;

    let output: Output<T> = Object.assign(fut.promise, {
        cancle(reason?: any) {
            controller.abort(reason);
            stream?.reader.cancel(reason);
            canceled = { reason };
        }
    });

    (async () => {
        try {
            let res = await readableStream;

            if (canceled) {
                res.cancel(canceled.reason);
                return fut.reject(canceled.reason);
            }

            stream = new Stream(res.getReader());
            let reader = new FrameDecoder(stream);
            let { data } = await reader.parseFrame();

            assert(data.type == "trailer", Error, `expected trailer`);
            assert(data.status == Status.Ok, Error, `trailer status: ${data.status}`);

            let de = new Decode(new Bytes(data.bytes));
            fut.resolve(decoder.call(de));
        } catch (error) {
            fut.reject(error)
            // output.cancle()
        }
    })();

    return output;
}
