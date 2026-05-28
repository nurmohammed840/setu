export * from "./cmd.ts";

export async function measurePerf<T>(f: () => T) {
    let name = f.name;
    console.time(name);
    let res = await f();
    console.timeEnd(name);
    return res;
}
