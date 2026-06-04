#!/usr/bin/env -S deno run -A --unsafely-ignore-certificate-errors
import { assertEquals } from "jsr:@std/assert";
import { HelloRequest, say_hello } from "./build/typescript/mod.ts";

let res = await say_hello({ req: new HelloRequest({ name: "Nur" }) });
assertEquals(res.message, "Hello Nur!");
