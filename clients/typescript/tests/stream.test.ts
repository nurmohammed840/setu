import { assertEquals, assertRejects } from "jsr:@std/assert";
import { encodeAsLastFrame } from "../src/setu/frame.writer.ts";
import { Stream, StreamReader } from "../src/utils/stream.ts";


function createStream(...s: number[][]) {
    const stream = new ReadableStream<Uint8Array<ArrayBuffer>>({
        start(c) {
            for (let buf of s) c.enqueue(new Uint8Array(buf));
            c.close();
        }
    });
    return new StreamReader(new Stream(stream.getReader()))
}

Deno.test("read byte", async () => {
    let de = createStream([], [4, 2], [], [42]);
    assertEquals(await de.readByte(), 4);
    assertEquals(await de.readByte(), 2);
    assertEquals(await de.readByte(), 42);
});

Deno.test("read bytes", async () => {
    let de = createStream([], [1, 2, 3], [], [4], [], [5]);

    assertEquals([...await de.readBytes(2)], [1, 2]);
    assertEquals([...await de.readBytes(0)], []);
    assertEquals([...await de.readBytes(1)], [3]);
    assertEquals([...await de.readBytes(0)], []);
    assertEquals([...await de.readBytes(2)], [4, 5]);

    await assertRejects(async () => await de.readBytes(1));
});

Deno.test("stream eof", async () => {
    let de = createStream([1], [2]);
    await assertRejects(async () => await de.readBytes(3));
});

Deno.test("encode as frame", () => {
    let raw = encodeAsLastFrame(new TextEncoder().encode("67"));
    assertEquals!([...raw], [0, 2, 54, 55, 2, 0]);
});