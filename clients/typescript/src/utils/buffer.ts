export class Buffer {
    #len = 0;
    #data: Uint8Array[] = [];
    constructor() { }

    append(buf: Uint8Array) {
        this.#len += buf.length;
        this.#data.push(buf)
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

