import { definePluginConfig } from "#rpp/config";
import type { Access, PluginEntry } from "#rpp/config";

import { kotlinTarget } from "./project.ts";
import type { KotlinTarget, WindowOptions } from "./project.ts";

export * from "./authoring/index.ts";
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
    const { kotlin, ...rest } = config;
    const options: WindowOptions = {
        ...rest,
        ...(kotlin === undefined
            ? {}
            : { kotlin: { packageName: kotlin.package, target: kotlinTarget(kotlin.target) } }),
    };
    return configure(options, {
        ...access,
        ...(kotlin === undefined ? {} : { outputs: { ...access?.outputs, kotlin: kotlin.output } }),
    });
};

export default window;
