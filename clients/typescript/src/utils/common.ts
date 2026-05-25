export function expected<T>(data?: T, err = "expected value"): T {
    assert(data != undefined, err);
    return data;
}


export function assert(expr: unknown, msg: (() => string) | string = ""): asserts expr {
    if (!expr) {
        throw new Error(typeof msg === "string" ? msg : msg());
    }
}

export function checkOverflow<T>(num: T, min: T, max: T) {
	if (num < min || num > max) {
		throw new Error(`expected ${min}..=${max}, got: ${num}`);
	}
}