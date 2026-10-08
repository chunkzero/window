import { isRef, refJson } from "../bind/handles.ts";
import { applyStyle } from "./styles.ts";
import { resolveTokens } from "./tokens.ts";
import type { ClickAction, Collection, Condition, Input, Items, Ref, Sprite, Text } from "../bind/handles.ts";
import type {
    AnvilInputElement,
    AnvilInputOptions,
    BoxOptions,
    CaseElement,
    CaseOptions,
    CollectionElement,
    CollectionOptions,
    FlexElement,
    FlexStyle,
    HandleJson,
    Hud,
    ItemElement,
    ItemOptions,
    LabelElement,
    LabelOptions,
    RegionElement,
    RegionOptions,
    SectionElement,
    SectionOptions,
    SlotElement,
    SlotOptions,
    SpriteElement,
    SpriteOptions,
    SpriteSlotElement,
    SpriteSlotOptions,
    SwitchElement,
    SwitchOptions,
    Window,
} from "./document.ts";
import type { ArtRef, SlotSection } from "./types.ts";

type Fields = Record<string, unknown>;

const TEXT_KEYS = [
    "x",
    "y",
    "width",
    "align",
    "color",
    "shadow",
    "bold",
    "italic",
    "underlined",
    "strikethrough",
    "obfuscated",
    "font",
    "small_caps",
    "layout",
    "debug_name",
    "style",
] as const;
const FLEX_KEYS = ["x", "y", "frame", "style", "theme", "children", "layout", "debug_name"] as const;
const SECTIONS: readonly SlotSection[] = ["container", "player", "hotbar"];
const PLACEMENT_KEYS = ["slots", "pattern", "transform"] as const;
const FIT_KEYS = ["overflow", "lines", "line_height"] as const;

function requireObject<T extends object>(value: T | undefined, label: string): T {
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
        throw new Error(`${label} must be an object`);
    }
    return value;
}

function requireName(value: unknown, label: string): void {
    if (typeof value !== "string" || value === "") {
        throw new Error(`${label} must be a non-empty string`);
    }
}

/** The `handle` field of an element reading `ref`, which must be a handle of one of `kinds`. */
function bound(ref: unknown, kinds: readonly string[], label: string): { handle: HandleJson } {
    if (!isRef(ref) || !kinds.includes(ref.kind)) {
        const expected = kinds.map((kind) => `\`${kind}\``).join(" or ");
        throw new Error(`${label} must be a ${expected} handle`);
    }
    return { handle: refJson(ref) as HandleJson };
}

function copyKnown(
    kind: string,
    opts: object | undefined,
    allowed: readonly string[],
    required: readonly string[] = [],
): Fields {
    const out: Fields = {};
    for (const [key, value] of Object.entries(requireObject(opts ?? {}, `${kind} options`))) {
        if (!allowed.includes(key)) {
            throw new Error(`${kind} does not accept option \`${key}\``);
        }
        if (value !== undefined) {
            out[key] = value;
        }
    }
    for (const key of required) {
        if (out[key] === undefined) {
            throw new Error(`${kind} requires option \`${key}\``);
        }
    }
    return out;
}

/** The `style` properties each element type accepts, by primitive. */
const STYLE_KINDS: Readonly<Record<string, string>> = {
    box: "box",
    label: "text",
    slot: "text",
    sprite: "item",
    sprite_slot: "item",
    region: "item",
    item: "item",
    collection: "collection",
    switch: "switch",
    section: "section",
};

function element(
    type: string,
    opts: object | undefined,
    allowed: readonly string[],
    required?: readonly string[],
): Fields {
    const out = copyKnown(type, opts, allowed, required);
    const kind = STYLE_KINDS[type];
    return { ...(kind === undefined ? out : applyStyle(out, kind, type)), type };
}

/** A root definition with its `style` applied and its vars resolved. */
export function root<T extends Window | Hud>(def: T, label: string): T {
    return resolveTokens(applyStyle({ ...def }, "section", label)) as T;
}

export function ui(def: Window): { windows: Window[] } {
    requireObject(def, "ui definition");
    if (def.name === undefined) {
        throw new Error("ui requires field `name`");
    }
    if (def.container === undefined) {
        throw new Error("ui requires field `container`");
    }
    return { windows: [root(def, "ui")] };
}

export function hud(def: Hud): { huds: Hud[] } {
    requireObject(def, "hud definition");
    if (def.name === undefined) {
        throw new Error("hud requires field `name`");
    }
    if ((def.width === undefined) !== (def.height === undefined)) {
        throw new Error("hud requires both `width` and `height`, or neither");
    }
    return { huds: [root(def, "hud")] };
}

/** A flexbox or grid whose `style` is built from the same styles as `<Box>`. */
export function box(opts?: BoxOptions): FlexElement {
    const out = element("box", opts, FLEX_KEYS);
    return { ...out, type: "flex", children: out.children ?? [] } as unknown as FlexElement;
}

/**
 * An inventory slot section: a grid with one track per slot that children auto-flow through. A `frame` draws around the
 * slot boxes, grown by `outset` (3 by default).
 */
export function section(kind: SlotSection, opts?: SectionOptions): SectionElement {
    if (!SECTIONS.includes(kind)) {
        throw new Error(`section kind must be one of ${SECTIONS.join(", ")}`);
    }
    const out = element("section", opts, ["frame", "outset", "claim", "flow", "style", "children", "debug_name"]);
    if (out.frame !== undefined && out.outset === undefined) {
        out.outset = 3;
    }
    return { ...out, section: kind, children: out.children ?? [] } as unknown as SectionElement;
}

/** Inline art drawn at its own size, or with a `sprite` handle a runtime sprite slot. */
export function image(art: ArtRef, opts?: SpriteOptions): SpriteElement;
export function image(bind: Sprite, opts: SpriteSlotOptions): SpriteSlotElement;
export function image(
    source: ArtRef | Sprite,
    opts?: SpriteOptions | SpriteSlotOptions,
): SpriteElement | SpriteSlotElement {
    if (isRef(source)) {
        return {
            ...element(
                "sprite_slot",
                opts,
                ["x", "y", "width", "height", "align", "layout", "style", "debug_name"],
                ["width", "height"],
            ),
            ...bound(source, ["sprite"], "image source"),
        } as unknown as SpriteSlotElement;
    }
    const out = element("sprite", opts, ["x", "y", "layout", "style", "debug_name"]);
    return { ...out, art: requireObject(source, "image art") } as unknown as SpriteElement;
}

/** An inventory region: the slots it covers route clicks to `onClick` and show its hitbox item. */
export function region(opts: RegionOptions = {}): RegionElement {
    const out = element("region", opts, [
        "on_click",
        "tooltip",
        "item_model",
        "width",
        "height",
        "layout",
        "style",
        "debug_name",
    ]);
    if ((out.width === undefined) !== (out.height === undefined)) {
        throw new Error("region sets `width` and `height` together, or neither to fill its box");
    }
    if (out.on_click !== undefined) {
        out.on_click = bound(
            out.on_click as ClickAction,
            ["action", "builtin", "toggle", "selection"],
            "region onClick",
        ).handle;
    }
    return out as unknown as RegionElement;
}

/** Static label text, or with a `text` handle dynamic text. */
export function text(content: string | Text, opts?: SlotOptions): LabelElement | SlotElement {
    return isRef(content) ? slot(content, opts) : label(content, opts);
}

/** Real item stacks in the slots it covers, rendered by an `items` handle. */
export function items(bind: Items, opts: ItemOptions = {}): ItemElement {
    const out = element("item", opts, [...PLACEMENT_KEYS, "layout", "style", "debug_name"]);
    return { ...out, ...bound(bind, ["items"], "items source") } as unknown as ItemElement;
}

/** Item stacks a `collection` handle supplies, one per cell. */
export function collection(bind: Collection, opts: CollectionOptions = {}): CollectionElement {
    const out = element("collection", opts, [
        "frame",
        "selected_sprite",
        ...PLACEMENT_KEYS,
        "action",
        "layout",
        "style",
        "debug_name",
    ]);
    return { ...out, ...bound(bind, ["collection"], "collection source") } as unknown as CollectionElement;
}

/**
 * The vanilla anvil rename field of an `anvil` window, bound by an `input` handle. The window's title must stay
 * static: no text slots, runtime sprite slots, or collection selected sprites, unless `experimentalAnvilUpdates` is
 * set.
 */
export function input(bind: Input, opts?: AnvilInputOptions): AnvilInputElement {
    return {
        ...element("anvil_input", opts, ["initial", "item_model", "debug_name"]),
        ...bound(bind, ["input"], "input source"),
    } as unknown as AnvilInputElement;
}

function label(text: string, opts?: LabelOptions): LabelElement {
    requireName(text, "label text");
    const fit = FIT_KEYS.find((key) => (opts as Fields | undefined)?.[key] !== undefined);
    if (fit !== undefined) {
        throw new Error(`label does not accept option \`${fit}\`; only bound text slots fit their content at runtime`);
    }
    return { ...element("label", opts, TEXT_KEYS), text } as unknown as LabelElement;
}

function slot(bind: Text, opts?: SlotOptions): SlotElement {
    return {
        ...element("slot", opts, [...TEXT_KEYS, ...FIT_KEYS]),
        ...bound(bind, ["text"], "text source"),
    } as unknown as SlotElement;
}

/** One case of `switchOn`: a box, laid out as a column by default, drawn while the switch selects `value`. */
export function switchCase(value: string, opts: CaseOptions = {}): CaseElement {
    requireName(value, "switch case value");
    const label = `switch case \`${value}\``;
    const out = applyStyle(
        copyKnown(label, opts, ["frame", "style", "theme", "children", "debug_name"]),
        "case",
        label,
    );
    const style: FlexStyle = { direction: "column", ...(out.style as FlexStyle | undefined) };
    return { ...out, type: "case", value, style, children: out.children ?? [] } as unknown as CaseElement;
}

/**
 * Stack `cases` in one box sized to the largest case; the runtime draws only the case `on` selects. `on` is a value or
 * selection handle, or a condition with `true` and `false` cases. `cases` maps each value to its case, or lists
 * `switchCase` elements. Cases may hold regions, items, and collections, which claim their slots only while their
 * case is drawn.
 */
export function switchOn(
    on: Ref<"value" | "selection"> | Condition,
    cases: Record<string, CaseOptions> | readonly CaseElement[],
    opts?: SwitchOptions,
): SwitchElement {
    const children = Array.isArray(cases)
        ? (cases as readonly CaseElement[]).map((c) => requireObject(c, "switchOn case"))
        : Object.entries(requireObject(cases as Record<string, CaseOptions>, "switchOn cases")).map(([value, body]) =>
              switchCase(value, body),
          );
    if (children.length === 0) {
        throw new Error("switchOn requires at least one case");
    }
    return {
        ...element("switch", { ...opts, children }, ["x", "y", "layout", "style", "children", "debug_name"]),
        ...bound(on, ["value", "selection", "flag", "toggle"], "switchOn source"),
    } as unknown as SwitchElement;
}
