export function takeBytes(N: number, buf: Uint8Array): [Uint8Array, Uint8Array] {
    if (N > buf.length) {
        throw new Error(`takeBytes(${N}) exceeds buffer length ${buf.length}`);
    }
    return [buf.subarray(0, N), buf.subarray(N)];
}

export class Bytes {
    static empty() {
        return new Bytes(new Uint8Array())
    }

    constructor(private data: Uint8Array) { }

    get length() {
        return this.remaining().length
    }

    isEmpty() {
        return this.length == 0
    }

    nextByte() {
        let [[byte], ptr] = takeBytes(1, this.data);
        this.data = ptr;
        return byte
    }

    take(len: number) {
        let [bytes, ptr] = takeBytes(len, this.data);
        this.data = ptr;
        return bytes;
    }

    remaining() {
        return this.data;
    }
}
