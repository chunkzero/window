/** Previews: a window, HUD, or full-screen window rendered to a PNG from handle values. See docs/AUTHORING.md. */
import type { Ref, Sprite } from "../bind/handles.ts";

/** A collection's previewed cells. */
export interface CollectionState {
    /** How many leading cells hold an item. Defaults to every cell. */
    count?: number;
    /** The cell drawing the collection's `selected` sprite. Defaults to none. */
    selected?: number;
}

/** A handle with a previewed value: every kind but actions. */
type Stateful = Ref<
    | "flag"
    | "toggle"
    | "value"
    | "selection"
    | "tint"
    | "fraction"
    | "text"
    | "sprite"
    | "items"
    | "collection"
    | "input"
>;

/** The value a preview gives the handle `H`. An indexed handle takes its members' values, nested as its shape. */
export type StateOf<H> = H extends { at(...index: infer I): infer M }
    ? I extends [number]
        ? readonly StateOf<M>[]
        : readonly (readonly StateOf<M>[])[]
    : H extends Ref<"text" | "input">
      ? string
      : H extends Ref<"flag" | "toggle" | "items">
        ? boolean
        : H extends Ref<"fraction">
          ? number
          : H extends Ref<"value" | "selection" | "tint", infer V>
            ? V
            : H extends Sprite<infer O>
              ? O
              : H extends Ref<"collection">
                ? CollectionState
                : never;

/** One handle and its previewed value, such as `[category, "gear"]` or `[label.at(1), "Gear"]`. */
export type StateEntry = readonly [handle: Stateful | { at(...index: never[]): Stateful }, value: unknown];

/** Checks each entry's value against its handle. */
type Checked<T extends readonly StateEntry[]> = {
    readonly [K in keyof T]: readonly [T[K][0], StateOf<T[K][0]>];
};

export interface PreviewOptions {
    /** The file stem of the PNG. Defaults to the target's name. */
    name?: string;
    /** Integer nearest-neighbour upscale from 1 to 16. Defaults to 1: one image pixel per GUI pixel. */
    scale?: number;
    /** Outline the active regions, which otherwise draw nothing. */
    outlines?: boolean;
}

/** A preview as the compiler reads it. */
export interface Preview {
    readonly name: string;
    readonly target: { readonly kind: "window" | "hud" | "fullscreen"; readonly name: string };
    readonly state: readonly { readonly handle: string; readonly at?: readonly number[]; readonly value: unknown }[];
    readonly scale?: number;
    readonly outlines?: boolean;
}

type Fields = Record<string, unknown>;

function isFields(value: unknown): value is Fields {
    return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * The one window, HUD, or full-screen window `value` defines: a `<Window>`, `<Hud>`, `<Fullscreen>`, `ui()`, `hud()` or
 * `fullscreen()` document, or a bare window or HUD definition.
 */
function target(value: unknown): Preview["target"] {
    if (isFields(value)) {
        for (const [list, kind] of [
            ["windows", "window"],
            ["huds", "hud"],
            ["fullscreens", "fullscreen"],
        ] as const) {
            const defs = value[list];
            if (Array.isArray(defs) && defs.length === 1 && isFields(defs[0]) && typeof defs[0]["name"] === "string") {
                return { kind, name: defs[0]["name"] };
            }
        }
        if (typeof value["name"] === "string") {
            return { kind: "container" in value ? "window" : "hud", name: value["name"] };
        }
    }
    throw new Error(
        "preview() takes one window, HUD, or full-screen window: a <Window>, <Hud>, <Fullscreen>, ui(), hud() or " +
            "fullscreen() document, or a window or HUD definition",
    );
}

/** The member values of an indexed handle, in `shape` order. */
function members(id: string, shape: readonly number[], values: unknown, at: readonly number[]): Preview["state"] {
    const size = shape[at.length];
    if (size === undefined) {
        return [{ handle: id, at, value: values }];
    }
    if (!Array.isArray(values) || values.length > size) {
        const where = at.map((i) => `[${i}]`).join("");
        throw new Error(`preview state of \`${id}${where}\` must be a list of at most ${size} values`);
    }
    return values.flatMap((value, i) => (value === undefined ? [] : members(id, shape, value, [...at, i])));
}

/**
 * A PNG of `target`, a window, HUD, or full-screen window, drawn with the handle values of `state`. List it in `defineWindows({ previews })`;
 * a build writes `<name>.png` into the plugin's `previews.output` directory. A handle without a value takes its
 * default: a toggle's or selection's `initial`, the first value of a `value`, the first palette entry of a `tint`,
 * `false` for a flag, 1 for a fraction, no text, and no sprite. Item slots draw as outlined placeholders.
 */
export function preview<const T extends readonly StateEntry[]>(
    target_: unknown,
    state?: T & Checked<T>,
    options: PreviewOptions = {},
): Preview {
    const surface = target(target_);
    const entries = ((state ?? []) as readonly StateEntry[]).flatMap(([handle, value]) => {
        const ref = handle as Ref;
        if (typeof ref !== "object" || ref === null || typeof ref.id !== "string") {
            throw new Error(`preview of \`${surface.name}\`: each state entry is \`[handle, value]\``);
        }
        return typeof (handle as { at?: unknown }).at === "function"
            ? members(ref.id, ref.shape ?? [], value, [])
            : [{ handle: ref.id, ...(ref.at === undefined ? {} : { at: ref.at }), value }];
    });
    return {
        name: options.name ?? surface.name,
        target: surface,
        state: entries,
        ...(options.scale === undefined ? {} : { scale: options.scale }),
        ...(options.outlines === undefined ? {} : { outlines: options.outlines }),
    };
}
