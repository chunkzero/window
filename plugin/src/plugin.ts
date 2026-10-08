import { components, definePlugin } from "rpp";
import type { Plugin } from "rpp";

import { generate } from "./project.ts";
import type { WindowOptions } from "./project.ts";

const plugin: Plugin<WindowOptions> = definePlugin<WindowOptions>({
    generate(ctx) {
        generate(ctx, components.load("compiler").exports.compile);
    },
});

export default plugin;
