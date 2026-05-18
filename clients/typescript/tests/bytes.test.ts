import { assertEquals, assertThrows } from "https://deno.land/std@0.224.0/assert/mod.ts";
import { takeBytes } from "../src/utils/bytes.ts";

Deno.test("take 2 bytes", () => {
    const buf = new Uint8Array([1, 2, 3, 4, 5]);
    const [a, b] = takeBytes(2, buf);

    assertEquals(Array.from(a), [1, 2]);
    assertEquals(Array.from(b), [3, 4, 5]);
    assertEquals(Array.from(buf), [1, 2, 3, 4, 5]);
});

Deno.test("take 0 bytes", () => {
    const [a, b] = takeBytes(0, new Uint8Array([1, 2, 3]));

    assertEquals(Array.from(a), []);
    assertEquals(Array.from(b), [1, 2, 3]);
});

Deno.test("take all bytes", () => {
    const [a, b] = takeBytes(3, new Uint8Array([1, 2, 3]));

    assertEquals(Array.from(a), [1, 2, 3]);
    assertEquals(Array.from(b), []);
});

Deno.test("take more bytes", () => {
    assertThrows(() => takeBytes(5, new Uint8Array([1, 2, 3])));
});

Deno.test("take is cheap", () => {
    const buf = new Uint8Array([1, 2, 3, 4]);
    const [a, b] = takeBytes(2, buf);

    buf[0] = 2
    buf[3] = 3;

    assertEquals(Array.from(a), [2, 2]);
    assertEquals(Array.from(b), [3, 3]);
});