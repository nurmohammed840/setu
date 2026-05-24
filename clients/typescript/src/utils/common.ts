export function expected<T>(data?: T, err = "expected value"): T {
    assert(data != undefined, err);
    return data;
}


export function assert(expr: unknown, msg: (() => string) | string = ""): asserts expr {
    if (!expr) {
        throw new Error(typeof msg === "string" ? msg : msg());
    }
}
