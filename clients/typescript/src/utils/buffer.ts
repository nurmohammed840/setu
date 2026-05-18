export class Buffer {
    #len = 0;
    #data: ArrayLike<number>[] = [];

    append(...bufs: ArrayLike<number>[]) {
        for (let buf of bufs) this.#len += buf.length;
        this.#data.push(...bufs)
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

