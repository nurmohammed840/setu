import { assertEquals } from "jsr:@std/assert";
import { encodeVarInt, decodeVarInt } from "../src/lipi/varint.ts";
import { Bytes } from "../src/utils/bytes.ts";

const maxNum = (bits: number) => (1n << BigInt(bits)) - 1n;


Deno.test("test varint encoding", () => {
    function check(num: bigint | number) {
        num = BigInt(num)
        let encoded = encodeVarInt(num);
        assertEquals!(num, decodeVarInt(new Bytes(encoded)));
    }

    for (let n = 0; n < 1000; n++) {
        check(n);
    }

    for (let exp = 0n; ; exp++) {
        const n = 2n ** exp;
        if (n > maxNum(64)) break;
        check(n);
    }

    for (let exp = 0n; ; exp++) {
        const n = 3n ** exp;
        if (n > maxNum(64)) break;
        check(n);
    }

    check(maxNum(64));
    check(maxNum(64) - 1n);

    check(maxNum(32));
    check(maxNum(32) + 1n);
    check(maxNum(32) - 1n);
});
