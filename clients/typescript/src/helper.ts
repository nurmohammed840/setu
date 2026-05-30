import { Encode, StructEncoder } from "./lipi/mod.ts";

export function Obj<T>(f: (s: StructEncoder, args: T) => void) {
    return function Struct(this: Encode, args: T) {
        let s = new StructEncoder(this);
        f(s, args);
        s.end()
    }
}
