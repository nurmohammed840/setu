import { assert, expected } from "./common.ts";
import { Buffer } from "./buffer.ts";
import { Bytes } from "./bytes.ts";
import { EndOfData } from "../errors.ts";

export class Stream {
    eos = false; // end of stream
    constructor(public reader: ReadableStreamDefaultReader<Uint8Array>) { }

    [Symbol.dispose]() {
        this.reader.cancel()
    }

    async read() {
        assert(!this.eos, EndOfData, "read after eos");
        const { done, value } = await this.reader.read();
        if (done) {
            this.eos = true;
            return;
        }
        return value;
    }

    async toBytes() {
        let chunk;
        let buf = new Buffer();
        while (chunk = await this.read())
            buf.push(chunk);

        return buf.data();
    }
}


export class StreamReader {
    data = Bytes.empty();
    constructor(public stream: Stream) { }

    [Symbol.dispose]() {
        this.stream[Symbol.dispose]()
    }

    async readBytes(len: number) {
        if (len == 0) {
            return new Uint8Array();
        }

        let data = await this.read();

        if (len <= data.length) {
            return data.take(len);
        }

        let buf = new Buffer();

        while (buf.len < len) {
            let data = await this.read();

            let remaining = len - buf.len;
            let takeN = Math.min(remaining, data.length);
            buf.push(data.take(takeN));
        }

        return buf.data()
    }

    async readByte() {
        let data = await this.read();
        return data.nextByte()
    }

    async read() {
        while (this.data.isEmpty()) {
            let bytes = expected(await this.stream.read(), "unexpected end of message");
            this.data = new Bytes(bytes);
        }
        return this.data;
    }
}

