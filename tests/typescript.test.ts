#!/usr/bin/env -S deno run -A --unsafely-ignore-certificate-errors
import { assertEquals } from "jsr:@std/assert";
import * as api from "./build/typescript/mod.ts";

// greeting
assertEquals(await api.say_hello({ input: { name: "Nur" } }), { message: "Hello Nur!" });

// common
assertEquals(await api.add({ a: 1, b: 2 }), 3);
assertEquals(await api.find_in_string({ input: "Löwe 老虎 Léopard Gepardi", pat: "é" }), 14);
assertEquals(await api.find_in_string({ input: "321", pat: "12" }), undefined);

await api.print({ msg: "Hello, World!" });

// stateful
assertEquals(await api.load(), undefined);
await api.store({ msg: "Top Secret! Shhhh...!" });
console.log(await api.load());
console.log("My IP:", await api.what_is_my_ip());

let ids = api.fetch_user_ids({ count: 3 });
async function run() {
    for await (const id of ids) {
        console.log("User ID:", id);
    }
}
run();
assertEquals(await ids.output(), "Bye!");