export function takeBytes(N: number, buf: Uint8Array): [Uint8Array, Uint8Array] {
    if (N > buf.length) {
        throw new Error(`takeBytes(${N}) exceeds buffer length ${buf.length}`);
    }
    return [buf.subarray(0, N), buf.subarray(N)];
}