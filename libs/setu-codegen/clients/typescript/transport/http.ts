import { ProtocolError } from "../errors.ts";
import { Input } from "./input.ts";
import { Output } from "./output.ts";
import { Timeout } from "../timeout.ts";
import { assert } from "../utils/common.ts";
import { Decoder } from "../lipi/decoder.ts";

export class RPC {
    static URL = new URL("/", "https://localhost:443");
    static TIMEOUT = Timeout.minute(2);

    static async call(id: number, input: Input, timeout: Timeout | null = RPC.TIMEOUT, url: URL = RPC.URL) {
        let headers: HeadersInit = {
            "content-type": "application/setu",
            "rpc-id": id.toString(),
        };

        let timer;
        if (timeout) {
            timer = setTimeout(() => input.controller.abort(), timeout.duration());
            headers["rpc-timeout"] = timeout.toString();
        }

        let res = await fetch(url, { method: "POST", headers, body: input.channel.stream, signal: input.controller.signal });

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

export function rpc<T>(id: number, { timeout, url }: Context, f: Decoder<T>) {
    let input = new Input();
    let output = Output<T>(input, RPC.call(id, input, timeout, url), f);
    return [input, output] as const;
}
