import { Status } from "../status.ts";
import { assert } from "../utils/common.ts";
import { StreamReader } from "../utils/stream.ts";

export class MaybeCompressed<T> {
    constructor(
        // @ts-ignore
        private isCompressed: boolean,
        public data: T
    ) { }
}

export type Frame = MessageFrame | TrailerFrame;

export interface MessageFrame {
    type: "message";
    bytes: Uint8Array;
}

export interface TrailerFrame {
    type: "trailer";
    status: Status;
    bytes: Uint8Array;
}

interface FrameHeaderArgs {
    lenSize: number,
    isCompressed?: boolean,
    trailer?: Status
}

export class FrameHeader {
    constructor(
        public isCompressed: boolean,
        public isTrailer: boolean,
        public lenSize: number,
        public code: number,
    ) { }

    static new = ({ lenSize, trailer, isCompressed }: FrameHeaderArgs) => new FrameHeader(
        !!isCompressed,
        trailer != undefined,
        lenSize - 1,
        trailer ?? 0,
    );

    static parse = (byte: number) => new FrameHeader(
        (byte & 0b1) === 0b1,
        (byte & 0b10) === 0b10,
        ((byte >> 2) & 0b11) + 1,
        byte >> 4,
    );

    encode() {
        return (
            (this.code << 4) |
            (this.lenSize << 2) |
            (+this.isTrailer << 1) |
            (+this.isCompressed)
        );
    }
}

export class LenBE {
    #buf = new ArrayBuffer(4);
    size: number;

    constructor(len: number) {
        new DataView(this.#buf).setUint32(0, len, false); // false = big-endian

        if (len <= 0xFF) this.size = 1;
        else if (len <= 0xFF_FF) this.size = 2;
        else if (len <= 0xFF_FF_FF) this.size = 3;
        else {
            assert(len <= 0xFF_FF_FF_FF, RangeError, () => `len: ${len} must fit in u32`);
            this.size = 4;
        }
    }

    asBytes(): Uint8Array {
        return new Uint8Array(this.#buf, 4 - this.size);
    }
}

export class FrameDecoder extends StreamReader {
    async parseFrame(): Promise<MaybeCompressed<Frame>> {
        let header = FrameHeader.parse(await this.readByte());
        let len = await this.parseLenBigEndian(header.lenSize);
        let bytes = await this.readBytes(len);

        return new MaybeCompressed(
            header.isCompressed,
            header.isTrailer
                ? { type: "trailer", status: Status.from(header.code), bytes }
                : { type: "message", bytes }
        );
    }

    async parseLenBigEndian(size: number) {
        let len = 0;
        for (let i = 0; i < size; i++) {
            len = (len << 8) | await this.readByte();
        }
        return 0;
    }
}
