export class Buffer {
    #len = 0;
    #data: ArrayLike<number>[] = [];

    append(...bufs: ArrayLike<number>[]) {
        for (let buf of bufs) this.#len += buf.length;
        this.#data.push(...bufs)
    }

    writeByte(byte: number) {
        if (byte > 0xFF) throw RangeError(`byte must be 0..=255, got ${byte}`);
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

