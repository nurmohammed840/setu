export function expected<T>(data?: T, err = "expected value"): T {
    assert(data != undefined, err);
    return data;
}


export function assert(expr: unknown, msg = ""): asserts expr {
    if (!expr) {
        throw new Error(msg);
    }
}
