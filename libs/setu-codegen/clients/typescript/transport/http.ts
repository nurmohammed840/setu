import { ProtocolError } from "../errors.ts";
import { Output, SSE } from "./output.ts";
import { Timeout } from "../timeout.ts";
import { assert } from "../utils/common.ts";
import { MPSC } from "../utils/mpsc.ts";
import { Decode } from "../lipi/decoder.ts";
import { Encode } from "../lipi/encoder.ts";
import { encodeFrame, encodeLastFrame } from "../setu/frame.writer.ts";
import { Status } from "../status.ts";
import { error } from "node:console";

export class RPC {
    static URL = new URL("/", "https://localhost:443");
    static TIMEOUT = Timeout.minute(2);

    static async call(
        id: number,
        body: BodyInit,
        conn: AbortController,
        timeout: Timeout | null = RPC.TIMEOUT,
        url: URL = RPC.URL
    ) {
        let headers: HeadersInit = {
            "content-type": "application/setu",
            "rpc-id": id.toString(),
        };

        let timer;
        if (timeout) {
            timer = setTimeout(() => conn.abort(), timeout.duration());
            headers["rpc-timeout"] = timeout.toString();
        }

        let res = await fetch(url, { method: "POST", headers, body, signal: conn.signal });

        clearTimeout(timer);

        if (!res.ok) {
            throw new Error(`${res.statusText}: ${await res.text()}`);
        }

        let contentType = res.headers.get("content-type");

        assert(contentType == "application/setu", ProtocolError, () => `unexpected content-type: ${contentType ?? "none"}`);
        assert(res.body, ProtocolError, "No response body");
        return res.body;
    }
}

export interface Context {
    url?: URL,
    timeout?: Timeout | null
}

export function rpc<T>(
    id: number, { timeout, url }: Context,
    input: (_: Encode) => void,
    output: (_: Decode) => T
): Output<T> {
    let conn = new AbortController();
    let body = encodeLastFrame(input);
    return Output(conn, RPC.call(id, body, conn, timeout, url), output);
}

export function sse<T, R>(
    id: number, { timeout, url }: Context,
    input: (_: Encode) => void,
    yielder: (_: Decode) => T,
    output: (_: Decode) => R,
): SSE<T, R> {
    let conn = new AbortController();
    let body = encodeLastFrame(input);
    return SSE(conn, RPC.call(id, body, conn, timeout, url), yielder, output);
}

export async function uni<T, R, O>(
    id: number, { timeout, url }: Context,
    input: (_: Encode) => void,
    output: (_: Decode) => O,
    send: (self: Encode, _: T) => void,
    final: (self: Encode, _: R) => void,
) {
    let conn = new AbortController();
    let writer = new MPSC<Uint8Array>();

    await writer.send(encodeFrame(input));

    let rpc = Output(conn, RPC.call(id, writer.stream, conn, timeout, url), output);

    return {
        [Symbol.dispose]() {
            writer.close();
        },

        send(value: T) {
            return writer.send(encodeFrame(e => send(e, value)));
        },
        async sendFinal(value: R) {
            await writer.send(encodeLastFrame(e => final(e, value)));
            writer.close();
        },
        async sendError(reason?: string) {
            writer.close();
        },

        cancle() {
            writer.close();
            rpc.cancle();
        },
        async output() {
            return await rpc;
        }
    }
}

export function bi() { }