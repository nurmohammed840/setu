import { bundle } from "jsr:@deno/emit";
import { measurePerf } from "./utils.ts";

let input = "./src/mod.ts"
let output = "./dist/index.js"

let { code, map } = await bundle(input, {
    // minify: true,
    compilerOptions: {
        sourceMap: true
    }
});

measurePerf("Done", () => {
    if (map) Deno.writeTextFileSync(`${output}.map`, map);
    Deno.writeTextFileSync(output, code)
});
