import { assertEquals, assert } from "jsr:@std/assert";
import { FrameDecoder, FrameHeader, LenBE } from "../src/setu/frame.ts";
import { encodeAsLastFrame } from "../src/setu/frame.writer.ts";
import { Status } from "../src/status.ts";
import { Stream } from "../src/utils/stream.ts";


Deno.test("Setu Frame Header", () => {
    let raw = FrameHeader.new({ lenSize: 4, status: Status.Cancelled }).encode();
    assertEquals(raw, 0b1_11_1_0);

    let h = FrameHeader.parse(raw);
    assert(!h.isCompressed);
    assert(h.isTrailer);
    assertEquals(h.lenSize, 4);
    assertEquals(h.code, 1);
});

Deno.test("LenBE basic", () => {
    assertEquals([...new LenBE(0x1234).asBytes()], [0x12, 0x34]);
    assertEquals([...new LenBE(0x123456).asBytes()], [0x12, 0x34, 0x56]);
    assertEquals([...new LenBE(0x12345678).asBytes()], [0x12, 0x34, 0x56, 0x78]);
});

Deno.test("last frame", async () => {
    let frame = encodeAsLastFrame(new TextEncoder().encode("67"));
    assertEquals([...frame], [2, 2, 54, 55]);

    let reader = new FrameDecoder(new Stream(ReadableStream.from([frame]).getReader()));
    let { data } = await reader.parseFrame();

    assert(data.type == "trailer");
    assert(data.status == Status.Ok);
    assertEquals([...data.bytes], [54, 55]);
});
