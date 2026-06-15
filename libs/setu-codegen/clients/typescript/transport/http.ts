import { ProtocolError } from "../errors.ts";
import { Output, SSE } from "./output.ts";
import { Timeout } from "../timeout.ts";
import { assert } from "../utils/common.ts";
import { Decode, Decoder } from "../lipi/decoder.ts";
import { Encode } from "../lipi/encoder.ts";
import { encodeTrailer } from "../setu/frame.writer.ts";

export class RPC {
    static URL = new URL("/", "https://localhost:443");
    static TIMEOUT = Timeout.minute(2);

    static async call(
        id: number,
        body: BodyInit,
        controller: AbortController,
        timeout: Timeout | null = RPC.TIMEOUT,
        url: URL = RPC.URL
    ) {
        let headers: HeadersInit = {
            "content-type": "application/setu",
            "rpc-id": id.toString(),
        };

        let timer;
        if (timeout) {
            timer = setTimeout(() => controller.abort(), timeout.duration());
            headers["rpc-timeout"] = timeout.toString();
        }

        let res = await fetch(url, { method: "POST", headers, body, signal: controller.signal });

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
    id: number,
    { timeout, url }: Context,
    encoder: (_: Encode) => void,
    decoder: (_: Decode) => T
): Output<T> {
    let controller = new AbortController();

    let e = new Encode();
    encoder(e);
    let body = encodeTrailer(e.data());

    return Output(controller, RPC.call(id, body, controller, timeout, url), decoder);
}

export function sse<T, R>(
    id: number,
    { timeout, url }: Context,
    encoder: (_: Encode) => void,
    yielder: (_: Decode) => T,
    output: (_: Decode) => R,
): SSE<T, R> {
    let controller = new AbortController();

    let e = new Encode();
    encoder(e);
    let body = encodeTrailer(e.data());

    return SSE(controller, RPC.call(id, body, controller, timeout, url), yielder, output);
}
