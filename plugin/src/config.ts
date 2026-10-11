import { definePluginConfig } from "rpp:config";
import type { Access, PluginEntry } from "rpp:config";

import { checkOptions, kotlinTarget } from "./project.ts";
import type { KotlinTarget, WindowOptions } from "./project.ts";

export type { KotlinTarget } from "./project.ts";

export type WindowConfig = Omit<WindowOptions, "kotlin"> & {
    /**
     * Generate Kotlin bindings into the `output` directory, relative to the project root. `target` picks the server
     * API window views bind to: `minestom` or `multistom` views take a `Player`; `agnostic` views take any
     * `WindowHost`. HUD views are the same for every target.
     */
    kotlin?: { package: string; output: string; target: KotlinTarget };
};

const configure: (options: WindowOptions, access?: Access) => PluginEntry = definePluginConfig<WindowOptions>("window");

/** Configure the Window plugin in `rpp.config.ts`. */
const window: (config?: WindowConfig, access?: Access) => PluginEntry = (config = {}, access) => {
    checkOptions(config);
    const { kotlin, ...rest } = config;
    const outputs = {
        ...(kotlin === undefined ? {} : { kotlin: kotlin.output }),
        ...(rest.previews === undefined ? {} : { previews: rest.previews.output }),
    };
    const options: WindowOptions = {
        ...rest,
        ...(kotlin === undefined
            ? {}
            : { kotlin: { packageName: kotlin.package, target: kotlinTarget(kotlin.target) } }),
    };
    return configure(options, {
        ...access,
        ...(Object.keys(outputs).length === 0 ? {} : { outputs: { ...access?.outputs, ...outputs } }),
    });
};

export default window;
