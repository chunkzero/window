import type {
    AnvilInputElement,
    AnvilInputOptions,
    ButtonElement,
    ButtonOptions,
    CaseElement,
    CaseOptions,
    ChoiceOptions,
    CollectionElement,
    CollectionOptions,
    ColumnElement,
    ColumnOptions,
    FlexElement,
    FlexOptions,
    FlexStyle,
    HotspotElement,
    HotspotOptions,
    Hud,
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
    SectionElement,
    SectionOptions,
    ShowOptions,
    SlotSection,
    SlotElement,
    SlotOptions,
    SlotRectsElement,
    SlotRectsOptions,
    SpriteElement,
    SpriteOptions,
    SpriteSlotElement,
    SpriteSlotOptions,
    SwitchElement,
    SwitchOptions,
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
    "layout",
] as const;
const LAYOUT_KEYS = ["x", "y", "gap", "padding", "align", "children", "layout"] as const;
const FLEX_KEYS = ["x", "y", "frame", "style", "children", "layout"] as const;
const SECTIONS: readonly SlotSection[] = ["container", "player", "hotbar"];
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
    if (def.name === undefined) {
        throw new Error("hud requires field `name`");
    }
    if ((def.width === undefined) !== (def.height === undefined)) {
        throw new Error("hud requires both `width` and `height`, or neither");
    }
    return { huds: [def] };
}

export function panel(opts: PanelOptions): PanelElement {
    return element(
        "panel",
        opts,
        ["frame", "width", "height", "x", "y", "padding", "children", "layout"],
        ["frame", "width", "height"],
    ) as unknown as PanelElement;
}

/** A CSS flexbox; `style` sets the container properties and children place themselves with `layout`. */
export function flex(opts?: FlexOptions): FlexElement {
    const out = element("flex", opts, FLEX_KEYS);
    return { ...out, children: out.children ?? [] } as unknown as FlexElement;
}

/** A CSS grid: `flex` with `style.display` set to `"grid"`. */
export function grid(opts?: FlexOptions): FlexElement {
    const out = element("grid", opts, FLEX_KEYS);
    const style = { ...(out.style as FlexStyle | undefined), display: "grid" };
    return { ...out, type: "flex", style, children: out.children ?? [] } as FlexElement;
}

/**
 * An inventory slot section: a grid with one track per slot that children auto-flow through. A `frame` draws around the
 * slot boxes, grown by `outset` (3 by default).
 */
export function section(kind: SlotSection, opts?: SectionOptions): SectionElement {
    if (!SECTIONS.includes(kind)) {
        throw new Error(`section kind must be one of ${SECTIONS.join(", ")}`);
    }
    const out = element("section", opts, ["frame", "outset", "claim", "flow", "children"]);
    if (out.frame !== undefined && out.outset === undefined) {
        out.outset = 3;
    }
    return { ...out, section: kind, children: out.children ?? [] } as unknown as SectionElement;
}

export function row(opts?: RowOptions): RowElement {
    return element("row", opts, LAYOUT_KEYS) as unknown as RowElement;
}

export function column(opts?: ColumnOptions): ColumnElement {
    return element("column", opts, LAYOUT_KEYS) as unknown as ColumnElement;
}

export function sprite(name: string, opts?: SpriteOptions): SpriteElement {
    requireName(name, "sprite name");
    return { ...element("sprite", opts, ["x", "y", "layout"]), name } as unknown as SpriteElement;
}

export function spriteSlot(name: string, opts: SpriteSlotOptions): SpriteSlotElement {
    requireName(name, "spriteSlot name");
    return {
        ...element(
            "sprite_slot",
            opts,
            ["x", "y", "width", "height", "align", "sprite", "layout", "index"],
            ["width", "height"],
        ),
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
        "hover_outline",
        "padding",
        "children",
        "layout",
    ]);
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
    const out = element("hotspot", opts, [
        "width",
        "height",
        "x",
        "y",
        ...PLACEMENT_KEYS,
        "tooltip",
        "states",
        "hover_outline",
        "layout",
    ]);
    if (out.tooltip === undefined && out.states === undefined) {
        throw new Error("hotspot requires option `tooltip` or `states`");
    }
    return { ...out, name } as unknown as HotspotElement;
}

export function item(name: string, opts: ItemOptions): ItemElement {
    requireName(name, "item name");
    const out = element("item", opts, [...PLACEMENT_KEYS, "cell_slot", "layout"]);
    if (out.cell_slot !== undefined) {
        if (out.slots !== undefined || out.pattern !== undefined || out.transform !== undefined) {
            throw new Error("item `cell_slot` cannot be combined with `slots`, `pattern`, or `transform`");
        }
        if (!Number.isInteger(out.cell_slot) || (out.cell_slot as number) < 1) {
            throw new Error("item `cell_slot` must be a positive integer");
        }
    }
    return { ...out, name } as unknown as ItemElement;
}

export function collection(name: string, opts: CollectionOptions): CollectionElement {
    requireName(name, "collection name");
    const out = element("collection", opts, ["frame", "selected_sprite", ...PLACEMENT_KEYS, "action", "layout"]);
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
    const out = element("slot_rects", opts, ["frame", "pattern", "transform", "claim", "layout"]);
    if (out.pattern === undefined && out.transform === undefined) {
        throw new Error("slotRects requires `pattern` or `transform`");
    }
    return { ...out, name } as unknown as SlotRectsElement;
}

export function repeater(name: string, opts: RepeaterOptions): RepeaterElement {
    requireName(name, "repeater name");
    const out = element("repeater", opts, [
        "frame",
        "pattern",
        "transform",
        "padding",
        "hover_outline",
        "children",
        "layout",
    ]);
    if (out.pattern === undefined && out.transform === undefined) {
        throw new Error("repeater requires `pattern` or `transform`");
    }
    return { ...out, name } as unknown as RepeaterElement;
}

const FIT_KEYS = ["overflow", "lines", "line_height"] as const;

export function label(text: string, opts?: LabelOptions): LabelElement {
    requireName(text, "label text");
    const fit = FIT_KEYS.find((key) => (opts as Fields | undefined)?.[key] !== undefined);
    if (fit !== undefined) {
        throw new Error(`label does not accept option \`${fit}\`; only bound text slots fit their content at runtime`);
    }
    return { ...element("label", opts, TEXT_KEYS), text } as unknown as LabelElement;
}

export function slot(name: string, opts?: SlotOptions): SlotElement {
    requireName(name, "slot name");
    return { ...element("slot", opts, [...TEXT_KEYS, ...FIT_KEYS, "index"]), name } as unknown as SlotElement;
}

function switchCase(name: string, value: string, opts: CaseOptions): CaseElement {
    requireName(value, `switch \`${name}\` case value`);
    const out = copyKnown(`switch \`${name}\` case \`${value}\``, opts, ["frame", "style", "children"]);
    const style: FlexStyle = { direction: "column", ...(out.style as FlexStyle | undefined) };
    return { ...out, type: "case", value, style, children: out.children ?? [] } as unknown as CaseElement;
}

/**
 * Stack `cases`, keyed by value, in one box sized to the largest case; the runtime draws only the case the binding
 * `name` returns. Cases are visual: art, labels, and text/icon slots, but no slot-bound controls.
 */
export function switchOn(name: string, cases: Record<string, CaseOptions>, opts?: SwitchOptions): SwitchElement {
    requireName(name, "switchOn name");
    const children = Object.entries(requireObject(cases, "switchOn cases")).map(([value, body]) =>
        switchCase(name, value, body),
    );
    if (children.length === 0) {
        throw new Error(`switchOn \`${name}\` requires at least one case`);
    }
    return { ...element("switch", opts, ["x", "y", "layout", "index"]), name, children } as unknown as SwitchElement;
}

/** Draw `opts.children` only while the Boolean binding `when` is true; their space is always reserved. */
export function show(when: string, opts: ShowOptions): SwitchElement {
    const { x, y, layout, index, ...box } = requireObject(opts, "show options");
    return switchOn(when, { true: box, false: {} }, { x, y, layout, index } as SwitchOptions);
}
