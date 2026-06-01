import { assertEquals, assert } from "jsr:@std/assert";
import { FrameHeader, LenBE } from "../src/setu/frame.ts";
import { Status } from "../src/status.ts";


Deno.test("Setu Frame Header", () => {
    let raw = FrameHeader.new({ lenSize: 4, trailer: Status.Cancelled }).encode();
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

