#!/usr/bin/env -S deno run -A --unsafely-ignore-certificate-errors
import { assertEquals } from "jsr:@std/assert";
import { say_hello, add } from "./build/typescript/mod.ts";

assertEquals(await say_hello({ input: { name: "Nur" } }), { message: "Hello Nur!" });
assertEquals(await add({ a: 1, b: 2 }), 3);
