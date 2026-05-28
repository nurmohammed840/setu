import { bundle } from "jsr:@deno/emit";
import { measurePerf } from "./utils.ts";
import { ensureDirSync } from "jsr:@std/fs/ensure-dir";

let input = "./src/mod.ts";
let outDir = "./javascript";

async function bundleJs() {
    let { code, map } = await bundle(input, {
        minify: false,
        compilerOptions: {
            sourceMap: true
        }
    });

    ensureDirSync(outDir);
    Deno.writeTextFileSync(`${outDir}/mod.js`, code)
    if (map) Deno.writeTextFileSync(`${outDir}/mod.js.map`, map);
}

measurePerf(bundleJs);