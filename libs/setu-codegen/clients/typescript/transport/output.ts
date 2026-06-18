import { Decode } from "../lipi/decoder.ts";
import { FrameDecoder } from "../setu/frame.ts";
import { Status } from "../status.ts";
import { Bytes } from "../utils/bytes.ts";
import { assert } from "../utils/common.ts";
import { Stream } from "../utils/stream.ts";

export interface Output<T> extends Promise<T> {
    cancle(reason?: any): void;
}

export function Output<T>(
    connection: AbortController,
    body: Promise<ReadableStream<Uint8Array>>,
    decoder: (_: Decode) => T
) {
    let fut = Promise.withResolvers<T>();
    let stream: Stream | undefined;
    let canceled: { reason?: any } | undefined;

    let output: Output<T> = Object.assign(fut.promise, {
        cancle(reason?: any) {
            stream?.reader.cancel(reason);
            connection.abort(reason);
            canceled = { reason };
        }
    });

    (async () => {
        try {
            let res = await body;
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
            fut.resolve(decoder(de));
        } catch (error) {
            fut.reject(error)
            // output.cancle()
        }
    })();

    return output;
}

export interface SSE<T, R> extends AsyncGenerator<T> {
    cancle(reason?: any): void;
    output(): Promise<R | undefined>;
}

export function SSE<T, R>(
    connection: AbortController,
    body: Promise<ReadableStream<Uint8Array>>,
    yielder: (_: Decode) => T,
    output: (_: Decode) => R,
): SSE<T, R> {
    let canceled: { reason?: any } | undefined;
    let stream: Stream | undefined;
    let fut = Promise.withResolvers<R | undefined>();

    let asyncIter = (async function* () {
        try {
            let res = await body;
            if (canceled) {
                res.cancel(canceled.reason);
                connection.abort(canceled.reason);
                return
            }

            stream = new Stream(res.getReader());
            let reader = new FrameDecoder(stream);

            while (true) {
                let { data } = await reader.parseFrame();
                let de = new Decode(new Bytes(data.bytes));
                if (data.type == "trailer") {
                    assert(data.status == Status.Ok, Error, `trailer status: ${data.status}`);
                    return fut.resolve(output(de));
                }
                yield yielder(de);
            }
        }
        finally {
            fut.resolve(undefined);
        }
    })();

    return Object.assign(asyncIter, {
        cancle(reason?: any) {
            canceled = { reason };
            asyncIter.return(undefined);

            stream?.reader.cancel(reason);
            connection.abort(reason);
        },
        output() {
            return fut.promise
        }
    })
}
