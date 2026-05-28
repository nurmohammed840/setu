import { Timeout } from "./timeout.ts";
import { Buffer } from "./utils/buffer.ts";
import { assert } from "./utils/common.ts";

const SETTINGS = {
    unaryTimeout: Timeout.minute(2)
};

interface Context {
    timeout?: Timeout
}

interface CallArgs {
    path: string | URL,
    call_id: number,
    body: BodyInit,
    ctx?: Context
}

export async function rpc({ path, call_id, body, ctx }: CallArgs) {
    ctx ??= { timeout: SETTINGS.unaryTimeout };

    let headers: HeadersInit = {
        "content-type": "application/setu",
        "rpc-id": call_id.toString(),
    };

    if (ctx.timeout) {
        headers["rpc-timeout"] = ctx.timeout.toString();
    }

    let res = await fetch(path, { method: "POST", headers, body });

    if (!res.ok) {
        throw new Error(`${res.statusText}: ${await res.text()}`);
    }

    let contentType = res.headers.get("content-type");

    assert(contentType == "application/setu", () => `unexpected content-type: ${contentType ?? "none"}`);
    assert(res.body, "No response body");

    return new HttpResponse(res.body.getReader());
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

// console.log(await rpc({
//     path: "https://127.0.0.1:4433/",
//     call_id: 7,
//     body: ""
// }));
