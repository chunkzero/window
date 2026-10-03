import type {
    AnvilInputElement,
    AnvilInputOptions,
    ButtonElement,
    ButtonOptions,
    ChoiceOptions,
    CollectionElement,
    CollectionOptions,
    ColumnElement,
    ColumnOptions,
    Hud,
    HotspotElement,
    HotspotOptions,
    ItemElement,
    ItemOptions,
    LabelElement,
    LabelOptions,
    PanelElement,
    PanelOptions,
    RepeaterElement,
    RepeaterOptions,
    RowElement,
    RowOptions,
    SlotElement,
    SlotOptions,
    SlotRectsElement,
    SlotRectsOptions,
    SpriteElement,
    SpriteOptions,
    SpriteSlotElement,
    SpriteSlotOptions,
    Theme,
    ToggleOptions,
    Window,
} from "./types.ts";

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
] as const;
const LAYOUT_KEYS = ["x", "y", "gap", "padding", "align", "children"] as const;
const PLACEMENT_KEYS = ["slots", "pattern", "transform"] as const;

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

function element(
    type: string,
    opts: object | undefined,
    allowed: readonly string[],
    required?: readonly string[],
): Fields {
    return { ...copyKnown(type, opts, allowed, required), type };
}

function requirePlacement(kind: string, out: Fields, extra: string): void {
    if (out.slots === undefined && out.pattern === undefined && out.transform === undefined) {
        throw new Error(`${kind} requires \`slots\`, \`pattern\`, \`transform\`${extra}`);
    }
}

function requireSize(kind: string, out: Fields): void {
    if (
        (out.width === undefined || out.height === undefined) &&
        out.pattern === undefined &&
        out.transform === undefined
    ) {
        throw new Error(`${kind} requires \`width\`/\`height\`, \`pattern\`, or \`transform\``);
    }
}

export function theme(def: Theme = {}): { theme: Theme } {
    return { theme: requireObject(def, "theme definition") };
}

export function ui(def: Window): { windows: Window[] } {
    requireObject(def, "ui definition");
    if (def.name === undefined) {
        throw new Error("ui requires field `name`");
    }
    if (def.container === undefined) {
        throw new Error("ui requires field `container`");
    }
    return { windows: [def] };
}

export function hud(def: Hud): { huds: Hud[] } {
    requireObject(def, "hud definition");
    for (const key of ["name", "width", "height"] as const) {
        if (def[key] === undefined) {
            throw new Error(`hud requires field \`${key}\``);
        }
    }
    return { huds: [def] };
}

export function panel(opts: PanelOptions): PanelElement {
    return element(
        "panel",
        opts,
        ["frame", "width", "height", "x", "y", "padding", "children"],
        ["frame", "width", "height"],
    ) as unknown as PanelElement;
}

export function row(opts?: RowOptions): RowElement {
    return element("row", opts, LAYOUT_KEYS) as unknown as RowElement;
}

export function column(opts?: ColumnOptions): ColumnElement {
    return element("column", opts, LAYOUT_KEYS) as unknown as ColumnElement;
}

export function sprite(name: string, opts?: SpriteOptions): SpriteElement {
    requireName(name, "sprite name");
    return { ...element("sprite", opts, ["x", "y"]), name } as unknown as SpriteElement;
}

export function spriteSlot(name: string, opts: SpriteSlotOptions): SpriteSlotElement {
    requireName(name, "spriteSlot name");
    return {
        ...element("sprite_slot", opts, ["x", "y", "width", "height", "align", "sprite"], ["width", "height"]),
        name,
    } as unknown as SpriteSlotElement;
}

export function button(name: string, opts?: ButtonOptions): ButtonElement {
    requireName(name, "button name");
    const out = element("button", opts, [
        "frame",
        "width",
        "height",
        "x",
        "y",
        ...PLACEMENT_KEYS,
        "default",
        "tooltip",
        "states",
        "padding",
        "children",
    ]);
    requireSize("button", out);
    return { ...out, name } as unknown as ButtonElement;
}

function statefulButton(
    kind: string,
    name: string,
    opts: ButtonOptions | undefined,
    firstState: string,
    secondState: string,
): ButtonElement {
    const states = requireObject(opts ?? {}, `${kind} options`).states;
    if (
        typeof states !== "object" ||
        states === null ||
        states[firstState] === undefined ||
        states[secondState] === undefined
    ) {
        throw new Error(`${kind} requires \`states.${firstState}\` and \`states.${secondState}\``);
    }
    return button(name, opts);
}

/** Create a two-state button for WindowScope.toggle. */
export function toggle(name: string, opts: ToggleOptions): ButtonElement {
    return statefulButton("toggle", name, opts, "on", "off");
}

/** Create one button in a mutually exclusive WindowScope.choice group. */
export function choice(name: string, opts: ChoiceOptions): ButtonElement {
    return statefulButton("choice", name, opts, "selected", "unselected");
}

export function hotspot(name: string, opts: HotspotOptions): HotspotElement {
    requireName(name, "hotspot name");
    const out = element("hotspot", opts, ["width", "height", "x", "y", ...PLACEMENT_KEYS, "tooltip", "states"]);
    requireSize("hotspot", out);
    if (out.tooltip === undefined && out.states === undefined) {
        throw new Error("hotspot requires option `tooltip` or `states`");
    }
    return { ...out, name } as unknown as HotspotElement;
}

export function item(name: string, opts: ItemOptions): ItemElement {
    requireName(name, "item name");
    const out = element("item", opts, [...PLACEMENT_KEYS, "cell_slot"]);
    if (out.cell_slot !== undefined) {
        if (out.slots !== undefined || out.pattern !== undefined || out.transform !== undefined) {
            throw new Error("item `cell_slot` cannot be combined with `slots`, `pattern`, or `transform`");
        }
        if (!Number.isInteger(out.cell_slot) || (out.cell_slot as number) < 1) {
            throw new Error("item `cell_slot` must be a positive integer");
        }
    } else {
        requirePlacement("item", out, ", or `cell_slot`");
    }
    return { ...out, name } as unknown as ItemElement;
}

export function collection(name: string, opts: CollectionOptions): CollectionElement {
    requireName(name, "collection name");
    const out = element("collection", opts, ["frame", "selected_sprite", ...PLACEMENT_KEYS, "action"]);
    requirePlacement("collection", out, "");
    return { ...out, name } as unknown as CollectionElement;
}

/**
 * Binds the vanilla anvil rename field in an `anvil` window. The window's title must stay static: no text slots,
 * unbound sprite slots, button state sprites, or collection selected sprites, unless `experimentalAnvilUpdates` is set.
 */
export function anvilInput(name: string, opts?: AnvilInputOptions): AnvilInputElement {
    requireName(name, "anvilInput name");
    return {
        ...element("anvil_input", opts, ["initial", "item_model"]),
        name,
    } as unknown as AnvilInputElement;
}

export function slotRects(name: string, opts: SlotRectsOptions): SlotRectsElement {
    requireName(name, "slotRects name");
    const out = element("slot_rects", opts, ["frame", "pattern", "transform", "claim"]);
    if (out.pattern === undefined && out.transform === undefined) {
        throw new Error("slotRects requires `pattern` or `transform`");
    }
    return { ...out, name } as unknown as SlotRectsElement;
}

export function repeater(name: string, opts: RepeaterOptions): RepeaterElement {
    requireName(name, "repeater name");
    const out = element("repeater", opts, ["frame", "pattern", "transform", "padding", "children"]);
    if (out.pattern === undefined && out.transform === undefined) {
        throw new Error("repeater requires `pattern` or `transform`");
    }
    return { ...out, name } as unknown as RepeaterElement;
}

export function label(text: string, opts?: LabelOptions): LabelElement {
    requireName(text, "label text");
    return { ...element("label", opts, TEXT_KEYS), text } as unknown as LabelElement;
}

export function slot(name: string, opts?: SlotOptions): SlotElement {
    requireName(name, "slot name");
    return { ...element("slot", opts, TEXT_KEYS), name } as unknown as SlotElement;
}
