import { Status } from "../status.ts";

export class MaybeCompressed<T> {
    isCompressed: boolean = false;
    constructor(private data: T) { }
}

export type Frame = { type: "message"; data: Uint8Array; } | { type: "trailer"; status: Status; bytes: Uint8Array; };

export class FrameHeader {
    constructor(
        public isCompressed: boolean,
        public isTrailer: boolean,
        public lenSize: number,
        public code: number,
    ) { }

    static new = (lenSize: number, trailer?: Status) => new FrameHeader(
        false,
        trailer != undefined,
        lenSize - 1,
        trailer ? Status.code(trailer) : 0,
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
