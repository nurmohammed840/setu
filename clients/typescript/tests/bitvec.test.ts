import { assert, assertEquals, } from "jsr:@std/assert";
import { bitvec, bitvecFrom, bitvecToBools, } from "../src/bitset.ts";

Deno.test("create_bit_set", () => {
    const bs1 = bitvecFrom([true, false]);
    const bs2 = bitvecFrom([true, false]);

    assertEquals(bs1.asBytes(), bs2.asBytes());

    const bs3 = bitvec(2);
    bs3.set(0);

    assertEquals(bs1.asBytes(), bs3.asBytes());
});

Deno.test("get_behavior", () => {
    const bs = bitvec(8);

    assertEquals(bs.get(8), undefined);

    bs.set(0);
    bs.set(7);

    assertEquals(bs.get(0), true);
    assertEquals(bs.get(7), true);

    assertEquals(bs.get(999), undefined);

    assert(!bs.isEmpty());

    bs.clear();

    assert(bs.isEmpty());
});

Deno.test("has_and_is_lsb_first", () => {
    const bs = bitvec(8);

    bs.set(0);
    assertEquals(Array.from(bs.asBytes()), [0b0000_0001]);

    bs.set(1);
    assertEquals(Array.from(bs.asBytes()), [0b0000_0011]);

    bs.set(7);
    assertEquals(Array.from(bs.asBytes()), [0b1000_0011]);

    assert(bs.has(0));
    assert(bs.has(1));
    assert(bs.has(7));

    assert(!bs.has(2));
});

Deno.test("insert_and_returns_old_value", () => {
    const bs = bitvec(16);

    assertEquals(bs.set(3), false);
    assert(bs.has(3));

    assertEquals(bs.set(3), true);
    assert(bs.has(3));
});

Deno.test("remove_and_returns_old_value", () => {
    const bs = bitvec(16);

    assertEquals(bs.remove(5), false);
    assert(!bs.has(5));

    assertEquals(bs.set(5), false);
    assert(bs.has(5));

    assertEquals(bs.remove(5), true);
    assert(!bs.has(5));
});

Deno.test("out_of_bounds_insert_and_remove", () => {
    const bs = bitvec(8);

    let err: unknown;

    try {
        bs.set(8);
    } catch (e) {
        err = e;
    }

    assert(err instanceof RangeError);

    assertEquals(bs.remove(8), undefined);

    assertEquals(bs.has(8), false);
    assertEquals(bs.has(9999), false);
});

Deno.test("bools_roundtrip", () => {
    const input = [
        true, false, true, true, false, false, false, true,
        false, true, false, false, true, false, true, false,
        true,
    ];

    const bs = bitvecFrom(input);
    const output = bitvecToBools(bs, input.length);

    assertEquals(output, input);
});

Deno.test("cross_byte_boundaries", () => {
    const bs = bitvec(16);

    bs.set(7);
    bs.set(8);
    bs.set(15);

    assert(bs.has(7));
    assert(bs.has(8));
    assert(bs.has(15));

    assertEquals(Array.from(bs.asBytes()), [0b1000_0000, 0b1000_0001]);
});