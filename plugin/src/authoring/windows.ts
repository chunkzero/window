import type { JSX } from "#rpp/jsx";

import type { Hud, Theme, Window } from "./types.ts";

/**
 * The project's themes, windows, and HUDs, as `window/index.ts(x)` default-exports them. TypeScript types every JSX
 * expression as `JSX.Element`, so `<Window>` and `<Hud>` entries are checked when the build reads them.
 */
export interface WindowsDefinition {
    /** Theme definitions, or `theme()` documents. */
    themes?: readonly (Theme | { theme: Theme })[];
    /** Windows, or `<Window>` and `ui()` documents. */
    windows?: readonly (Window | { windows: Window[] } | JSX.Element)[];
    /** HUDs, or `<Hud>` and `hud()` documents. */
    huds?: readonly (Hud | { huds: Hud[] } | JSX.Element)[];
}

/**
 * Declares every theme, window, and HUD of the pack. Default-export it from `window/index.ts` or `window/index.tsx`;
 * the other files under `window/` are then ordinary modules.
 */
export function defineWindows(definition: WindowsDefinition): WindowsDefinition {
    return definition;
}
