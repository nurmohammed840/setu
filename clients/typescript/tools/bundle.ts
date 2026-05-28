import { bundle, BundleOptions } from "jsr:@deno/emit";
import { measurePerf, runCmd } from "./utils/mod.ts";
import { ensureDirSync } from "jsr:@std/fs/ensure-dir";

let input = "./src/mod.ts";
let outDir = "./javascript";

let option: BundleOptions = {
    minify: false,
    compilerOptions: {
        sourceMap: false
    }
};

async function bundleJs() {
    let { code, map } = await bundle(input, option);

    ensureDirSync(outDir);
    Deno.writeTextFileSync(`${outDir}/mod.js`, code)
    if (map) Deno.writeTextFileSync(`${outDir}/mod.js.map`, map);
    await runCmd("tsc");
}

measurePerf(bundleJs);