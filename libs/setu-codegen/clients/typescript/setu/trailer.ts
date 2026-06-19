import { Decode, StructDecoder } from "../lipi/decoder.ts";
import { Encode, StructEncoder } from "../lipi/encoder.ts";

export class Trailer {
    static error(reason?: string) {
        return new Trailer(reason)
    }

    static encode = function Struct(this: Encode, z: Trailer) {
        let _ = this;
        StructEncoder(_, [
            [1, z.error, _.Str]
        ]);
    }

    static decoder = function Struct(this: Decode): Trailer {
        let _ = this;
        return StructDecoder(_, [
            [1, "error", _.Str, 0],
        ]);
    }

    constructor(public error?: string) { }
}
