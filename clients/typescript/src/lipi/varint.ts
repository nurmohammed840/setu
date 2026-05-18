import { Bytes } from "../utils/bytes.ts";

export function encodeVarInt(num: bigint | number) {
    num = BigInt(num);
    if (num < 0n) throw new RangeError(`expected unsigned number: found ${num}`);

    let buf = [];
    while (num > 0b111_1111) {
        buf.push(Number((num & 0xFFn) | 0b1000_0000n));
        num >>= 7n;
    }
    buf.push(Number(num));
    return new Uint8Array(buf);
}

export function decodeVarInt(bytes: Bytes): bigint {
    let result = 0n;
    let shift = 0n;

    while (true) {
        let byte = BigInt(bytes.nextByte());
        if (shift == 63n && byte >= 2) throw new Error("invalid variable-length integer");

        if ((byte & 0b1000_0000n) == 0n) return result | (byte << shift);

        result |= (byte & 0b111_1111n) << shift; // low-order 7 bits of value
        shift += 7n;
    }
}
