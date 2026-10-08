import type { Var } from "./tokens.ts";
import type { Art, Hud, Theme, Window } from "./types.ts";

/**
 * The project's themes, windows, and HUDs, as `window/index.ts(x)` default-exports them. Lists may nest, so a
 * default-exported fragment or array can be listed as one entry.
 */
export interface WindowsDefinition {
    /** Theme definitions, or `theme()` documents. */
    themes?: readonly unknown[];
    /** `<Window>` and `ui()` documents, or window definitions. */
    windows?: readonly unknown[];
    /** `<Hud>` and `hud()` documents, or HUD definitions. */
    huds?: readonly unknown[];
    /**
     * Runtime sprites by name: inline art Kotlin selects through the generated `WindowSprite`, such as
     * `{ lamp_on: texture("window/lamp_on.png") }`. A `sprite` handle's `only` names some of them. Vars in them take
     * their default values.
     */
    sprites?: Readonly<Record<string, Art | Var<Art>>>;
}

/**
 * Checks each entry of the list `L` against `D`. TypeScript types every JSX expression as the empty `JSX.Element`,
 * so an entry typed `{}` passes here and is checked when the build reads it.
 */
type Entries<L, D> = { readonly [K in keyof L]: Entry<L[K], D> };
type Entry<E, D> = {} extends E ? E : E extends readonly unknown[] ? Entries<E, D> : E extends D ? E : D;

type Checked<T extends WindowsDefinition> = {
    sprites?: Readonly<Record<string, Art | Var<Art>>>;
    themes?: Entries<T["themes"], Theme | { theme: Theme }>;
    windows?: Entries<T["windows"], Window | { windows: Window[] }>;
    huds?: Entries<T["huds"], Hud | { huds: Hud[] }>;
};

/**
 * Declares every theme, window, and HUD of the pack. Default-export it from `window/index.ts` or `window/index.tsx`;
 * the other files under `window/` are then ordinary modules.
 */
export function defineWindows<const T extends WindowsDefinition>(definition: T & Checked<T>): WindowsDefinition {
    return definition;
}
