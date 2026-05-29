import { Input } from "./input.ts";
import { Timeout } from "./timeout.ts";
import { Buffer } from "./utils/buffer.ts";
import { assert } from "./utils/common.ts";

export class RPC {
    static URL = new URL("/", "https://localhost:443");
    static TIMEOUT = Timeout.minute(2);

    static async call(id: number, body: BodyInit, timeout: Timeout | null = RPC.TIMEOUT, url: URL = RPC.URL) {
        let headers: HeadersInit = {
            "content-type": "application/setu",
            "rpc-id": id.toString(),
        };

        if (timeout) {
            headers["rpc-timeout"] = timeout.toString();
        }

        let res = await fetch(url, { method: "POST", headers, body });

        if (!res.ok) {
            throw new Error(`${res.statusText}: ${await res.text()}`);
        }

        let contentType = res.headers.get("content-type");

        assert(contentType == "application/setu", () => `unexpected content-type: ${contentType ?? "none"}`);
        assert(res.body, "No response body");

        return res.body;
    }
}

export interface Context {
    url?: URL,
    timeout?: Timeout | null
}

export function rpc(id: number, { timeout, url }: Context) {
    let input = new Input();
    let output = RPC.call(id, input.channel.stream, timeout, url);
    return [input, output] as const;
}

export class HttpResponse {
    eos = false; // end of stream
    constructor(private reader: ReadableStreamDefaultReader<Uint8Array>) { }

    [Symbol.dispose]() {
        this.reader.cancel()
    }

    async read() {
        assert(!this.eos, "read after eos");
        const { done, value } = await this.reader.read();
        if (done) {
            this.eos = true;
            return;
        }
        return value;
    }

    async toBytes() {
        let chunk;
        let buf = new Buffer();
        while (chunk = await this.read())
            buf.append(chunk);

        return buf.data();
    }
}
