export function expected<T>(data?: T, err = new Error("expected value")): T {
    if (!data) {
        throw err
    }
    return data;
}

