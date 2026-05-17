import { concat } from "jsr:@std/bytes/concat";

export class Buffer {
    data: Uint8Array[] = [];
    constructor() { }

    bytes() {
        return concat(this.data)
    }
}

