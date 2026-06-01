export function expected<T>(data?: T, err = "expected value"): T {
    assert(data != undefined, err, TypeError);
    return data;
}


export function assert(expr: unknown, msg: (() => string) | string = "", err: ErrorConstructor = Error): asserts expr {
    if (!expr) {
        throw new err(typeof msg === "string" ? msg : msg());
    }
}

export function checkOverflow<T>(num: T, min: T, max: T) {
	if (num < min || num > max) {
        throw new RangeError(`expected ${min}..=${max}, got: ${num}`);
	}
}