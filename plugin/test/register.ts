// Lets `node --test` load the plugin's `.tsx` sources: oxc compiles their JSX against `rpp:jsx`, which resolves to
// the SDK `rpp codegen` writes. Node strips the types of plain `.ts` files itself.
import { readFileSync } from "node:fs";
import { registerHooks } from "node:module";
import { fileURLToPath } from "node:url";

import { transformSync } from "oxc-transform";

const runtime = new URL("../.rpp/sdk/jsx.ts", import.meta.url).href;

registerHooks({
    resolve(specifier, context, nextResolve) {
        if (specifier === "rpp:jsx" || specifier === "rpp:jsx/jsx-runtime") {
            return { url: runtime, shortCircuit: true };
        }
        return nextResolve(specifier, context);
    },
    load(url, context, nextLoad) {
        if (!url.startsWith("file:") || !url.endsWith(".tsx")) {
            return nextLoad(url, context);
        }
        const path = fileURLToPath(url);
        const result = transformSync(path, readFileSync(path, "utf8"), {
            jsx: { runtime: "automatic", importSource: "rpp:jsx" },
        });
        if (result.errors.length > 0) {
            throw new Error(`${path}: ${result.errors.map((error) => error.message).join("\n")}`);
        }
        return { format: "module", source: result.code, shortCircuit: true };
    },
});
