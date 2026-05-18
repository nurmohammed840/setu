import { assertEquals, assert, assertRejects } from "jsr:@std/assert";
import { FrameHeader, FrameDecoder, LenBE } from "../src/setu/frame.ts";
import { Status } from "../src/status.ts";
import { HttpResponse } from "../src/http.transport.ts";
import { encodeAsFrame } from "../src/setu/frame.writer.ts";


Deno.test("Setu Frame Header", () => {
    let raw = FrameHeader.new({ lenSize: 4, trailer: Status.Cancelled }).encode();
    assertEquals(raw, 0b1_11_1_0);

    let h = FrameHeader.parse(raw);
    assert(!h.isCompressed);
    assert(h.isTrailer);
    assertEquals(h.lenSize, 4);
    assertEquals(h.code, 1);
});

function createStream(...s: number[][]) {
    const stream = new ReadableStream<Uint8Array<ArrayBuffer>>({
        start(c) {
            for (let buf of s) c.enqueue(new Uint8Array(buf));
            c.close();
        }
    });
    return new FrameDecoder(new HttpResponse(stream.getReader()))
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
    assertEquals([...await de.readBytes(3)], [3, 4, 5]);
});

Deno.test("stream eof", async () => {
    let de = createStream([1], [2]);
    await assertRejects(async () => await de.readBytes(3));
});

Deno.test("LenBE basic", () => {
    assertEquals([...new LenBE(0x1234).asBytes()], [0x12, 0x34]);
    assertEquals([...new LenBE(0x123456).asBytes()], [0x12, 0x34, 0x56]);
    assertEquals([...new LenBE(0x12345678).asBytes()], [0x12, 0x34, 0x56, 0x78]);
});

Deno.test("encode as frame", () => {
    let raw = encodeAsFrame(new TextEncoder().encode("67"));
    assertEquals!([...raw], [0, 2, 54, 55, 2, 0]);
});