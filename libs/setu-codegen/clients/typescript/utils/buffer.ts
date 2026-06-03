import { checkOverflowUint } from "./common.ts";

export class Buffer {
    #len = 0;
    #data: ArrayLike<number>[] = [];

    append(buf: ArrayLike<number>) {
        this.#len += buf.length;
        this.#data.push(buf)
    }

    appendMany(...bufs: ArrayLike<number>[]) {
        for (let buf of bufs) this.#len += buf.length;
        this.#data.push(...bufs)
    }

    writeByte(byte: number) {
        checkOverflowUint(byte, 8);
        this.#len += 1;
        this.#data.push([byte]);
    }

    get len() {
        return this.#len;
    }

    data() {
        let b = new Uint8Array(this.#len);
        let i = 0;
        for (let buf of this.#data) {
            b.set(buf, i);
            i += buf.length;
        }
        return b;
    }
}

