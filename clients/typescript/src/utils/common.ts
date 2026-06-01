type ErrorClass = new (message?: string) => Error;
type ErrorMessage = (() => string) | string;

export function expected<T>(data?: T, err = "expected value"): T {
    assert(data !== undefined, TypeError, err);
    return data;
}

export function assert(expr: unknown, err: ErrorClass = Error, msg: ErrorMessage = ""): asserts expr {
    if (expr) return;
    const e = new err(typeof msg === "string" ? msg : msg());
    //@ts-ignore
    Error.captureStackTrace(e, assert);
    throw e;
}

export function checkOverflow<T>(num: T, min: T, max: T) {
    if (num < min || num > max) {
        throw new RangeError(`expected ${min}..=${max}, got: ${num}`);
    }
}

function isLittleEndian() {
    const buf = new ArrayBuffer(4);
    new Uint32Array(buf)[0] = 0x11_22_33_44;
    return new Uint8Array(buf)[0] == 0x44;
}

export const IS_LITTLE_ENDIAN = isLittleEndian();