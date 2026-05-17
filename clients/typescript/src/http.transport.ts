import { Timeout } from "./timeout.ts";

const SETTINGS = {
    unaryTimeout: Timeout.minute(2)
};

interface Context {
    timeout?: Timeout
}

export async function rpc(service: string | URL, call_id: number, ctx: Context = { timeout: SETTINGS.unaryTimeout }) {
    let headers: HeadersInit = {
        "content-type": "application/setu",
        "rpc-id": call_id.toString(),
    };

    if (ctx.timeout) {
        headers["rpc-timeout"] = ctx.timeout.toString();
    }

    let rpc = await fetch(service, { method: "POST", headers });

    if (!rpc.ok) {
        throw new Error(`${rpc.statusText}: ${await rpc.text()}`);
    }

    let contentType = rpc.headers.get("content-type");
    if (contentType != "application/setu") {
        throw new Error(
            `unexpected content-type: ${contentType ?? "none"}`
        );
    }

    return rpc;
}

console.log(await rpc("https://127.0.0.1:4433/", 7));
