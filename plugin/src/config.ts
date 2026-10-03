import { definePluginConfig } from "#rpp/config";
import type { Access, PluginEntry } from "#rpp/config";

import type { WindowOptions } from "./project.ts";

export * from "./authoring/index.ts";

export type WindowConfig = Omit<WindowOptions, "kotlinPackage"> & {
    /** Generate Kotlin bindings into the `output` directory, relative to the project root. */
    kotlin?: { package: string; output: string };
};

const configure: (options: WindowOptions, access?: Access) => PluginEntry = definePluginConfig<WindowOptions>("window");

/** Configure the Window plugin in `rpp.config.ts`. */
const window: (config?: WindowConfig, access?: Access) => PluginEntry = (config = {}, access) => {
    const { kotlin, ...rest } = config;
    const options: WindowOptions = { ...rest, ...(kotlin === undefined ? {} : { kotlinPackage: kotlin.package }) };
    return configure(options, {
        ...access,
        ...(kotlin === undefined ? {} : { outputs: { ...access?.outputs, kotlin: kotlin.output } }),
    });
};

export default window;
