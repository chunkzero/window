import { components, definePlugin } from "rpp";
import type { Plugin } from "rpp";

import { generate } from "./project.ts";
import type { WindowOptions } from "./project.ts";

const plugin: Plugin<WindowOptions> = definePlugin<WindowOptions>({
    generate(ctx) {
        const { compile, renderPreviews } = components.load("compiler").exports;
        generate(ctx, compile, renderPreviews);
    },
});

export default plugin;
