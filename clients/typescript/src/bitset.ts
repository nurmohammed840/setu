import { assert } from "./utils/common.ts";

export interface BitSetRead {
    capacity(): number;
    isEmpty(): boolean;
    has(index: number): boolean;
    get(index: number): boolean | undefined;
}

export interface BitSetWrite {
    clear(): void;
    set(index: number): boolean;
    remove(index: number): boolean | undefined;
}

export class BitVec implements BitSetRead, BitSetWrite {
    private bytes: Uint8Array;

    constructor(lenOrBytes: number | Uint8Array) {
        if (typeof lenOrBytes === "number") {
            this.bytes = new Uint8Array(boolPackedLen(lenOrBytes));
        } else {
            this.bytes = lenOrBytes;
        }
    }

    asBytes() {
        return this.bytes;
    }

    capacity() {
        return this.bytes.length * 8;
    }

    isEmpty() {
        return this.bytes.every(slot => slot == 0);
    }

    has(index: number) {
        return this.get(index) ?? false;
    }

    get(index: number) {
        const slotIdx = Math.floor(index / 8);
        const mask = 1 << (index % 8);

        const slot = this.bytes[slotIdx];
        if (slot === undefined) {
            return undefined;
        }

        return (slot & mask) !== 0;
    }

    clear(): void {
        this.bytes.fill(0);
    }

    set(index: number) {
        const slotIdx = Math.floor(index / 8);
        const mask = 1 << (index % 8);

        assert(slotIdx < this.bytes.length, () => `Out of bounds slot index: ${slotIdx}`);

        const oldValue = (this.bytes[slotIdx]! & mask) !== 0;
        this.bytes[slotIdx]! |= mask;

        return oldValue;
    }

    remove(index: number) {
        const slotIdx = Math.floor(index / 8);
        const mask = 1 << (index % 8);

        if (slotIdx >= this.bytes.length) {
            return undefined;
        }

        const oldValue = (this.bytes[slotIdx]! & mask) !== 0;
        this.bytes[slotIdx]! &= ~mask;

        return oldValue;
    }
}


export function boolPackedLen(len: number) {
    assert(len >= 0, () => `length ${len} cannot be negative`);
    return Math.floor((len + 7) / 8);
}

export function bitvec(len: number | Uint8Array) {
    return new BitVec(len);
}

export function bitvecFrom(bools: boolean[]) {
    const bv = bitvec(bools.length);

    for (let i = 0; i < bools.length; i++) {
        if (bools[i]) {
            bv.set(i);
        }
    }

    return bv;
}

export function bitvecToBools(len: number, bitvec: Uint8Array | BitVec) {
    const bv = bitvec instanceof BitVec ? bitvec : new BitVec(bitvec);

    const out: boolean[] = [];

    for (let i = 0; i < len; i++) {
        out.push(bv.has(i));
    }

    return out;
}