#!/usr/bin/env -S deno run -A --unsafely-ignore-certificate-errors
import { assertEquals } from "jsr:@std/assert";
import { say_hello, add, find_in_string, print } from "./build/typescript/mod.ts";

// greeting
assertEquals(await say_hello({ input: { name: "Nur" } }), { message: "Hello Nur!" });

// common
assertEquals(await add({ a: 1, b: 2 }), 3);
assertEquals(await find_in_string({ input: "Löwe 老虎 Léopard Gepardi", pat: "é" }), 14);
assertEquals(await find_in_string({ input: "321", pat: "12" }), undefined);

await print({ msg: "Hello, World!" });