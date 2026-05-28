export function measurePerf<T>(label: string, f: () => T) {
    console.time(label);
    let res = f();
    console.timeEnd(label);
    return res;
}