import { hud, ui } from "./elements.ts";
import type {
    AutoLength,
    ButtonDefault,
    ContainerKind,
    Edges,
    Element,
    FixedLength,
    FlexAlign,
    FlexJustify,
    FlexStyle,
    GridFlow,
    GridLine,
    Hud as HudDef,
    HudChannel,
    Insets,
    ItemLayout,
    Length,
    SlotList,
    SlotPatternInput,
    SlotRectClaim,
    SlotSection,
    TextAlign,
    Tooltip,
    Window as WindowDef,
} from "./types.ts";

export type Window = WindowDef;
export type Hud = HudDef;

type Fields = Record<string, unknown>;

/** Anything JSX can render as a child. Strings become labels; arrays and fragments flatten. */
export type Child = JsxNode | string | number | boolean | null | undefined | readonly Child[];

/** Text styling. Containers pass it down through `text`; the nearest setting wins. */
export interface TextProps {
    color?: string;
    shadow?: boolean;
    bold?: boolean;
    italic?: boolean;
    underlined?: boolean;
    strikethrough?: boolean;
    obfuscated?: boolean;
    font?: string;
    smallCaps?: boolean;
    align?: TextAlign;
}

/** Layout of an element inside a box or a slot section. */
export interface ItemProps {
    /** Flex grow factor; `true` is 1. */
    grow?: number | boolean;
    shrink?: number;
    basis?: Length;
    alignSelf?: FlexAlign;
    justifySelf?: FlexAlign;
    margin?: Edges;
    /** Take the element out of flow; position it with `top`/`right`/`bottom`/`left`. */
    absolute?: boolean;
    top?: AutoLength;
    right?: AutoLength;
    bottom?: AutoLength;
    left?: AutoLength;
    /** Grid area size in tracks; in a section, one track is one slot. `[columns, rows]` or a column count. */
    span?: number | [number, number];
    /** Zero-based grid position; in a section, the slot column and row. */
    at?: [number, number];
    /** Raw one-based grid lines; override `span` and `at`. */
    col?: GridLine;
    row?: GridLine;
    /** Pixel offset applied after layout to the element and its subtree. */
    translate?: [number, number];
}

export interface BoxProps extends ItemProps {
    frame?: string;
    /** Explicit pixel position; inside another box this positions the box absolutely. */
    x?: number;
    y?: number;
    display?: "flex" | "grid";
    direction?: "row" | "column" | "row-reverse" | "column-reverse";
    wrap?: boolean;
    justify?: FlexJustify;
    align?: FlexAlign;
    alignContent?: FlexJustify;
    gap?: FixedLength | [FixedLength, FixedLength];
    padding?: Edges<FixedLength>;
    width?: Length;
    height?: Length;
    minWidth?: AutoLength;
    minHeight?: AutoLength;
    maxWidth?: AutoLength;
    maxHeight?: AutoLength;
    aspectRatio?: number;
    columns?: FlexStyle["columns"];
    rows?: FlexStyle["rows"];
    flow?: GridFlow;
    text?: TextProps;
    children?: Child;
}

export interface TextElementProps extends TextProps, ItemProps {
    /** Dynamic text slot name. Without it, the children are static label text. */
    bind?: string;
    /** Fixed width; dynamic text without one grows to fill its box. */
    width?: number;
    x?: number;
    y?: number;
    children?: Child;
}

export interface SpriteProps extends ItemProps {
    name: string;
    x?: number;
    y?: number;
}

export interface IconProps extends ItemProps {
    /** Runtime sprite slot name. */
    bind: string;
    /** Slot size in pixels; a number is square. */
    size: number | [number, number];
    /** Fixed theme sprite; the slot then needs no Kotlin binding. */
    sprite?: string;
    align?: TextAlign;
    x?: number;
    y?: number;
}

export interface StateProps {
    itemModel?: string;
    sprite?: string;
    tooltip?: string | Tooltip;
}

interface SlotSource {
    /** Raw slot refs; the control is then not placed on the section grid. */
    slots?: SlotList;
    /** A slot pattern; in a section it is relative to the control's grid area. */
    pattern?: SlotPatternInput;
}

export interface ButtonProps extends ItemProps, SlotSource {
    name: string;
    frame?: string;
    tooltip?: string | Tooltip;
    states?: Record<string, StateProps>;
    /** Close the window on click. */
    close?: boolean;
    padding?: number;
    gap?: FixedLength;
    text?: TextProps;
    children?: Child;
}

export interface ToggleProps extends Omit<ButtonProps, "states"> {
    on: StateProps;
    off: StateProps;
}

export interface ChoiceProps extends Omit<ButtonProps, "states"> {
    selected: StateProps;
    unselected: StateProps;
}

export interface TabsProps extends ItemProps {
    /** Prefix of every tab's choice name: `<name>_<value>`. */
    name: string;
    frame?: string;
    /** Sprite drawn behind an unselected tab. */
    sprite?: string;
    /** Sprite drawn behind the selected tab. */
    selectedSprite?: string;
    itemModel?: string;
    text?: TextProps;
    children?: Child;
}

export interface TabProps {
    value: string;
    tooltip?: string | Tooltip;
    children?: Child;
}

export interface HotspotProps extends ItemProps, SlotSource {
    name: string;
    tooltip?: string | Tooltip;
    states?: Record<string, StateProps>;
}

export interface ItemSlotProps extends ItemProps, SlotSource {
    name: string;
    /** One-based slot inside the enclosing repeater cell. */
    cellSlot?: number;
}

export interface CollectionProps extends ItemProps, SlotSource {
    name: string;
    frame?: string;
    /** Sprite marking the selected cell. */
    selected?: string;
    /** `false` for a display-only collection. */
    action?: boolean;
    /** Width in slots; defaults to the full section width. */
    columns?: number;
    rows?: number;
}

export interface RepeaterProps extends ItemProps {
    name: string;
    /** Size of one cell in slots. */
    cell: [number, number];
    columns: number;
    rows: number;
    frame?: string;
    padding?: number;
    text?: TextProps;
    children?: Child;
}

export interface SlotsProps extends ItemProps {
    name: string;
    frame?: string;
    claim?: SlotRectClaim;
}

export interface AnvilInputProps {
    name: string;
    initial?: string;
    itemModel?: string;
}

export interface SectionProps {
    frame?: string;
    /** How far the frame extends past the slot boxes. Defaults to 3 when a frame is set. */
    outset?: Insets;
    /** Claim for slots no child owns. Defaults to "unowned". */
    claim?: SlotRectClaim;
    flow?: GridFlow;
    text?: TextProps;
    children?: Child;
}

export interface WindowProps {
    name: string;
    container: ContainerKind;
    bleed?: Insets;
    frame?: string;
    text?: TextProps;
    children?: Child;
}

export type HudAnchor =
    | "top-left"
    | "top"
    | "top-right"
    | "left"
    | "center"
    | "right"
    | "bottom-left"
    | "bottom"
    | "bottom-right";

export interface HudProps extends Omit<BoxProps, keyof ItemProps | "x" | "y" | "frame" | "width" | "height"> {
    width?: number;
    height?: number;
    name: string;
    channel?: HudChannel;
    bleed?: Insets;
    frame?: string;
    /** Screen point the HUD is pinned to, through the generated core shaders. */
    anchor?: HudAnchor | { x: number; y: number };
    /** GUI-pixel nudge from the anchor. */
    offset?: [number, number];
    /** See HudShader.source_bottom. */
    sourceBottom?: number;
}

type Component<P> = (props: P) => JsxNode;
type RenderNode = Element | { windows: WindowDef[] } | { huds: HudDef[] } | TabNode;
export type JsxNode = RenderNode | readonly JsxNode[];

interface TabNode {
    type: "tab";
    value: string;
    tooltip: string | Tooltip | undefined;
    children: Element[];
}

/** The classic JSX factory. Set `@jsx h` and `@jsxFrag Fragment` on separate pragma lines. */
export function h<P>(type: Component<P>, props: P | null, ...children: Child[]): JsxNode {
    return type({ ...(props ?? ({} as P)), ...(children.length === 0 ? {} : { children }) } as P);
}

export declare namespace h {
    namespace JSX {
        type Element = JsxNode;
        interface ElementChildrenAttribute {
            children: unknown;
        }
        interface IntrinsicElements {}
    }
}

export function Fragment(props: { children?: Child }): RenderNode[] {
    return renderNodes(props.children);
}

function flatten(child: Child, out: (RenderNode | string)[]): void {
    if (child === null || child === undefined || typeof child === "boolean") {
        return;
    }
    if (Array.isArray(child)) {
        for (const c of child as readonly Child[]) {
            flatten(c, out);
        }
    } else if (typeof child === "number" || typeof child === "string") {
        const last = out[out.length - 1];
        if (typeof last === "string") {
            out[out.length - 1] = last + String(child);
        } else {
            out.push(String(child));
        }
    } else {
        out.push(child as RenderNode);
    }
}

function renderNodes(children: Child): RenderNode[] {
    const out: (RenderNode | string)[] = [];
    flatten(children, out);
    return out.flatMap((c) => {
        if (typeof c !== "string") {
            return [c];
        }
        const text = c.trim();
        return text === "" ? [] : [{ type: "label", text } as Element];
    });
}

function nodes(children: Child): Element[] {
    return renderNodes(children).map((node) => {
        if (!("type" in node) || node.type === "tab") {
            throw new Error("Window/Hud roots belong in fragments; <Tab> belongs inside <Tabs>");
        }
        return node;
    });
}

function text(children: Child): string {
    const out: (RenderNode | string)[] = [];
    flatten(children, out);
    if (out.some((c) => typeof c !== "string")) {
        throw new Error("<Text> children must be text");
    }
    return out.join("");
}

type Defined<T> = { [K in keyof T]?: Exclude<T[K], undefined> };

function clean<T extends Fields>(fields: T): Defined<T> {
    const out: Fields = {};
    for (const [key, value] of Object.entries(fields)) {
        if (value !== undefined) {
            out[key] = value;
        }
    }
    return out as Defined<T>;
}

function nonEmpty<T extends Fields>(fields: T): Defined<T> | undefined {
    const out = clean(fields);
    return Object.keys(out).length === 0 ? undefined : out;
}

const TEXT_KEYS: [keyof TextProps, string][] = [
    ["color", "color"],
    ["shadow", "shadow"],
    ["bold", "bold"],
    ["italic", "italic"],
    ["underlined", "underlined"],
    ["strikethrough", "strikethrough"],
    ["obfuscated", "obfuscated"],
    ["font", "font"],
    ["smallCaps", "small_caps"],
    ["align", "align"],
];

function textFields(props: TextProps): Fields {
    const out: Fields = {};
    for (const [from, to] of TEXT_KEYS) {
        if (props[from] !== undefined) {
            out[to] = props[from];
        }
    }
    return out;
}

/** Fills unset text fields without modifying reusable children. */
function cascade<T>(node: T, style: TextProps | undefined): T {
    if (style === undefined) {
        return node;
    }
    const defaults = textFields(style);
    const visit = (el: Fields): Fields => {
        const result = { ...el };
        if (el.type === "label" || el.type === "slot") {
            const fontSet = el.font !== undefined || el.small_caps !== undefined;
            for (const [key, value] of Object.entries(defaults)) {
                if (el[key] === undefined && !((key === "font" || key === "small_caps") && fontSet)) {
                    result[key] = value;
                }
            }
        }
        for (const key of ["children", "windows", "huds"]) {
            if (Array.isArray(el[key])) {
                result[key] = (el[key] as Fields[]).map(visit);
            }
        }
        return result;
    };
    return (Array.isArray(node) ? node.map((el) => visit(el as Fields)) : visit(node as Fields)) as T;
}

function spanOf(span: ItemProps["span"]): [number | undefined, number | undefined] {
    if (span === undefined) {
        return [undefined, undefined];
    }
    return typeof span === "number" ? [span, undefined] : span;
}

function line(at: number | undefined, span: number | undefined): GridLine | undefined {
    if (at !== undefined) {
        return span === undefined ? at + 1 : { start: at + 1, span };
    }
    return span === undefined ? undefined : { span };
}

function layout(props: ItemProps): Fields {
    const [columns, rows] = spanOf(props.span);
    const absolute = props.absolute === true ? ("absolute" as const) : undefined;
    const item: ItemLayout | undefined = nonEmpty({
        grow: typeof props.grow === "boolean" ? Number(props.grow) : props.grow,
        shrink: props.shrink,
        basis: props.basis,
        align_self: props.alignSelf,
        justify_self: props.justifySelf,
        margin: props.margin,
        position: absolute,
        top: props.top,
        right: props.right,
        bottom: props.bottom,
        left: props.left,
        column: props.col ?? line(props.at?.[0], columns),
        row: props.row ?? line(props.at?.[1], rows),
        translate: props.translate,
    });
    return item === undefined ? {} : { layout: item };
}

function state(props: StateProps): Fields {
    return clean({ item_model: props.itemModel, sprite: props.sprite, tooltip: props.tooltip });
}

function states(map: Record<string, StateProps> | undefined): Fields | undefined {
    if (map === undefined) {
        return undefined;
    }
    return Object.fromEntries(Object.entries(map).map(([key, value]) => [key, state(value)]));
}

function requireName(name: unknown, component: string): void {
    if (typeof name !== "string" || name === "") {
        throw new Error(`<${component}> requires a non-empty \`name\``);
    }
}

/** A flexbox (or grid) container. Auto-sized boxes fill their parent's content box. */
export function Box(props: BoxProps): Element {
    const style: FlexStyle | undefined = nonEmpty({
        display: props.display,
        direction: props.direction,
        wrap: props.wrap,
        justify: props.justify,
        align: props.align,
        align_content: props.alignContent,
        gap: props.gap,
        padding: props.padding,
        width: props.width,
        height: props.height,
        min_width: props.minWidth,
        min_height: props.minHeight,
        max_width: props.maxWidth,
        max_height: props.maxHeight,
        aspect_ratio: props.aspectRatio,
        columns: props.columns,
        rows: props.rows,
        auto_flow: props.flow,
    });
    const node = clean({
        type: "flex",
        frame: props.frame,
        x: props.x,
        y: props.y,
        style,
        children: nodes(props.children),
        ...layout(props),
    });
    return cascade(node as Element, props.text);
}

/** A horizontal box whose children are vertically centered by default. */
export function Row(props: BoxProps): Element {
    return Box({ align: "center", ...props, direction: "row" });
}

export function Column(props: BoxProps): Element {
    return Box({ ...props, direction: "column" });
}

export function Grid(props: BoxProps): Element {
    return Box({ ...props, display: "grid" });
}

/** A box that centers its children on both axes. */
export function Center(props: BoxProps): Element {
    return Box({ justify: "center", align: "center", ...props });
}

/** Flexible empty space; in a section it skips `span` slots. */
export function Spacer(props: ItemProps): Element {
    return Box({ grow: 1, ...props });
}

/** Static label text, or dynamic text with `bind`. */
export function Text(props: TextElementProps): Element {
    const style = textFields(props);
    const common = clean({ width: props.width, x: props.x, y: props.y, ...style, ...layout(props) });
    if (props.bind !== undefined) {
        requireName(props.bind, "Text bind");
        return { type: "slot", name: props.bind, ...common } as Element;
    }
    const value = text(props.children);
    if (value === "") {
        throw new Error("<Text> requires text children or `bind`");
    }
    return { type: "label", text: value, ...common } as Element;
}

/** A static theme sprite. */
export function Sprite(props: SpriteProps): Element {
    requireName(props.name, "Sprite");
    return clean({ type: "sprite", name: props.name, x: props.x, y: props.y, ...layout(props) }) as Element;
}

/** A runtime sprite slot, drawn above button and selection backgrounds. */
export function Icon(props: IconProps): Element {
    requireName(props.bind, "Icon bind");
    const [width, height] = typeof props.size === "number" ? [props.size, props.size] : props.size;
    return clean({
        type: "sprite_slot",
        name: props.bind,
        width,
        height,
        sprite: props.sprite,
        align: props.align,
        x: props.x,
        y: props.y,
        ...layout(props),
    }) as Element;
}

function section(kind: SlotSection, props: SectionProps): Element {
    const node = clean({
        type: "section",
        section: kind,
        frame: props.frame,
        outset: props.outset ?? (props.frame === undefined ? undefined : 3),
        claim: props.claim,
        flow: props.flow,
        children: nodes(props.children),
    });
    return cascade(node as Element, props.text);
}

/** The opened container's slot grid. Children auto-flow through it, one track per slot. */
export function Container(props: SectionProps): Element {
    return section("container", props);
}

/** The player's 9x3 main inventory grid. */
export function Player(props: SectionProps): Element {
    return section("player", props);
}

/** The player's 9x1 hotbar grid. */
export function Hotbar(props: SectionProps): Element {
    return section("hotbar", props);
}

/** Content children are centered in a row filling the button. */
export function Button(props: ButtonProps): Element {
    requireName(props.name, "Button");
    const kids = nodes(props.children);
    const content =
        kids.length === 0
            ? []
            : [
                  Row({
                      justify: "center",
                      gap: props.gap ?? 2,
                      children: kids,
                      text: { align: props.text?.align ?? "center" },
                  }),
              ];
    const node = clean({
        type: "button",
        name: props.name,
        frame: props.frame,
        tooltip: props.tooltip,
        states: states(props.states),
        default: props.close === true ? ("close" satisfies ButtonDefault) : undefined,
        padding: props.padding,
        slots: props.slots,
        pattern: props.pattern,
        children: content,
        ...layout(props),
    });
    return cascade(node as Element, props.text);
}

/** A two-state button bound with WindowScope.toggle. */
export function Toggle(props: ToggleProps): Element {
    const { on, off, ...rest } = props;
    return Button({ ...rest, states: { on, off } });
}

/** One button of a WindowScope.choice group. */
export function Choice(props: ChoiceProps): Element {
    const { selected, unselected, ...rest } = props;
    return Button({ ...rest, states: { selected, unselected } });
}

/** One tab of a `<Tabs>` group. */
export function Tab(props: TabProps): TabNode {
    return { type: "tab", value: props.value, tooltip: props.tooltip, children: nodes(props.children) };
}

/** A choice group: each `<Tab value>` becomes the choice `<name>_<value>`. */
export function Tabs(props: TabsProps): Element[] {
    requireName(props.name, "Tabs");
    const tabs = renderNodes(props.children);
    return tabs.map((tab) => {
        if (!("type" in tab) || tab.type !== "tab") {
            throw new Error("<Tabs> children must be <Tab> elements");
        }
        const t = tab;
        const label = t.tooltip ?? t.children.flatMap((c) => (c.type === "label" ? [c.text] : [])).join(" ");
        const tooltip = label === "" ? undefined : label;
        const base = clean({ itemModel: props.itemModel, tooltip });
        return Choice({
            ...props,
            name: `${props.name}_${t.value}`,
            selected: clean({ ...base, sprite: props.selectedSprite }),
            unselected: clean({ ...base, sprite: props.sprite }),
            children: t.children,
        });
    });
}

export function Hotspot(props: HotspotProps): Element {
    requireName(props.name, "Hotspot");
    return clean({
        type: "hotspot",
        name: props.name,
        tooltip: props.tooltip,
        states: states(props.states),
        slots: props.slots,
        pattern: props.pattern,
        ...layout(props),
    }) as Element;
}

/** A real item stack. */
export function Item(props: ItemSlotProps): Element {
    requireName(props.name, "Item");
    return clean({
        type: "item",
        name: props.name,
        cell_slot: props.cellSlot,
        slots: props.slots,
        pattern: props.pattern,
        ...layout(props),
    }) as Element;
}

/** A scrolling item collection; by default it spans the full section width. */
export function Collection(props: CollectionProps): Element {
    requireName(props.name, "Collection");
    const full =
        props.columns === undefined && props.span === undefined && props.at === undefined && props.col === undefined;
    return clean({
        type: "collection",
        name: props.name,
        frame: props.frame,
        selected_sprite: props.selected,
        action: props.action,
        slots: props.slots,
        pattern: props.pattern,
        ...layout({
            span: [props.columns ?? 1, props.rows ?? 1],
            ...props,
            ...(full ? { col: { start: 1, end: -1 } } : {}),
        }),
    }) as Element;
}

/**
 * A grid of `columns` x `rows` cells, each a button of `cell` slots. `<Item cellSlot>` children stay in the cell;
 * the other children are centered in a column filling it.
 */
export function Repeater(props: RepeaterProps): Element {
    requireName(props.name, "Repeater");
    const all = nodes(props.children);
    const items = all.filter((c) => c.type === "item");
    const kids = all.filter((c) => c.type !== "item");
    const node = clean({
        type: "repeater",
        name: props.name,
        frame: props.frame,
        padding: props.padding,
        pattern: {
            kind: "grid",
            section: "container",
            x: 0,
            y: 0,
            columns: props.columns,
            rows: props.rows,
            cell_width: props.cell[0],
            cell_height: props.cell[1],
        },
        children: [
            ...items,
            ...(kids.length === 0 ? [] : [Column({ justify: "center", align: "center", children: kids })]),
        ],
        ...layout(props),
    });
    return cascade(node as Element, props.text);
}

/** Draws `frame` on, and optionally claims, a block of `span` slots. */
export function Slots(props: SlotsProps): Element {
    requireName(props.name, "Slots");
    const { span, ...rest } = props;
    const [columns, rows] = spanOf(span);
    return clean({
        type: "slot_rects",
        name: props.name,
        frame: props.frame,
        claim: props.claim,
        pattern: { kind: "rect", section: "container", x: 0, y: 0, width: columns ?? 1, height: rows ?? 1 },
        ...layout(rest),
    }) as Element;
}

/** The anvil's native rename field, bound as a text input. */
export function AnvilInput(props: AnvilInputProps): Element {
    requireName(props.name, "AnvilInput");
    return clean({
        type: "anvil_input",
        name: props.name,
        initial: props.initial,
        item_model: props.itemModel,
    }) as Element;
}

/** The title strip above the container grid, centering its children. */
export function Header(props: BoxProps): Element {
    return Box({ justify: "center", align: "center", gap: 4, ...props, x: 0, y: 0, width: 176, height: 17 });
}

/** A container window. Default-export it from a `.tsx` file under `window/`. */
export function Window(props: WindowProps): { windows: WindowDef[] } {
    requireName(props.name, "Window");
    const def = clean({
        name: props.name,
        container: props.container,
        bleed: props.bleed,
        frame: props.frame,
        children: nodes(props.children),
    }) as WindowDef;
    return cascade(ui(def), props.text);
}

const ANCHORS: Record<HudAnchor, [number, number]> = {
    "top-left": [0, 0],
    top: [0.5, 0],
    "top-right": [1, 0],
    left: [0, 0.5],
    center: [0.5, 0.5],
    right: [1, 0.5],
    "bottom-left": [0, 1],
    bottom: [0.5, 1],
    "bottom-right": [1, 1],
};

/**
 * A HUD whose children lay out as a column by default. Without `width`/`height` it sizes to its content.
 * `anchor` pins it to a screen point with the generated core shaders (requires `hudShaders`).
 */
export function Hud(props: HudProps): { huds: HudDef[] } {
    requireName(props.name, "Hud");
    const { name, channel, bleed, frame, anchor, offset, sourceBottom, width, height, ...box } = props;
    const fixed = typeof width === "number" && typeof height === "number";
    if (!fixed && (width !== undefined || height !== undefined)) {
        throw new Error("<Hud> `width` and `height` must both be pixel numbers, or both be omitted");
    }
    const root = Box({ direction: "column", ...box });
    let shader: HudDef["shader"];
    if (anchor !== undefined) {
        const [x, y] = typeof anchor === "string" ? ANCHORS[anchor] : [anchor.x, anchor.y];
        shader = clean({
            source_bottom: sourceBottom,
            origin: { x, y },
            anchor: { x, y },
            x: offset?.[0],
            y: offset?.[1],
        });
    }
    const def = clean({
        name,
        channel,
        bleed,
        frame,
        shader,
        width: fixed ? width : undefined,
        height: fixed ? height : undefined,
        children: [root],
    }) as HudDef;
    return hud(def);
}
