import { assertEquals, assert } from "jsr:@std/assert";
import { FrameHeader } from "../src/setu/frame.ts";
import { Status } from "../src/status.ts";


Deno.test("Setu Frame Header", () => {
    let raw = FrameHeader.new(4, Status.Cancelled).encode();
    assertEquals(raw, 0b1_11_1_0);

    let h = FrameHeader.parse(raw);
    assert(!h.isCompressed);
    assert(h.isTrailer);
    assertEquals(h.lenSize, 4);
    assertEquals(h.code, 1);
});