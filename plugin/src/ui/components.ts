import type { Child as JsxChild } from "rpp:jsx";

import { hud, region, ui } from "./elements.ts";
import { isRef, refJson } from "./handles.ts";
import { STYLE_KEYS, cascade, flexStyle, layout, textFields, withStyle } from "./styles.ts";
import type {
    BoxStyle,
    CaseStyle,
    CollectionStyle,
    HudStyle,
    ItemStyle,
    SectionStyle,
    StyleValue,
    SwitchStyle,
    TextProps,
    TextStyle,
} from "./styles.ts";
import type { VarTheme } from "./tokens.ts";
import type { ClickAction, Collection, Condition, Flag, Ref } from "./handles.ts";
import type { Input as InputHandle, Items as ItemsHandle } from "./handles.ts";
import type { Sprite as SpriteHandle, Text as TextHandle, Toggle as ToggleHandle } from "./handles.ts";
import type {
    ArtRef,
    CaseElement,
    ContainerKind,
    Element,
    GridFlow,
    Hud as HudDef,
    HudChannel,
    Insets,
    SlotList,
    SlotPatternInput,
    SlotRectClaim,
    SlotSection,
    TextAlign,
    TextOverflow,
    Tooltip,
    Window as WindowDef,
} from "./types.ts";

export type Window = WindowDef;
export type Hud = HudDef;

type Fields = Record<string, unknown>;

/** Anything JSX can render as a child. Strings become labels; arrays and fragments flatten. */
export type Child = JsxChild;

export type { TextProps };

/** The authored name of a primitive, shown in errors and in the inspector. */
export interface DebugProps {
    debugName?: string;
}

function debug(props: DebugProps): Fields {
    return props.debugName === undefined ? {} : { debug_name: props.debugName };
}

/** Layout of an element inside a box or a slot section. */
export type ItemProps = ItemStyle;

export interface BoxProps extends BoxStyle, DebugProps {
    /** Explicit pixel position; inside another box this positions the box absolutely. */
    x?: number;
    y?: number;
    /** Styles from `create`, merged under the explicit props. */
    style?: StyleValue<BoxStyle>;
    /** Overrides vars for this box and everything inside it. */
    theme?: VarTheme;
    children?: Child;
}

export interface TextElementProps extends TextStyle, DebugProps {
    style?: StyleValue<TextStyle>;
    /** Dynamic text: a `text` handle. Without it, the children are static label text. */
    bind?: TextHandle;
    /** Fixed width; dynamic text without one grows to fill its box. */
    width?: number;
    x?: number;
    y?: number;
    /** Shortens bound text wider than its width at runtime; requires `bind`. */
    overflow?: TextOverflow;
    /** Wraps bound text onto at most this many lines, ellipsizing the last; requires `bind`. */
    lines?: number;
    /** Distance between the tops of wrapped lines in pixels; defaults to 9. */
    lineHeight?: number;
    children?: Child;
}

interface ImagePlacement extends ItemProps, DebugProps {
    x?: number;
    y?: number;
    style?: StyleValue<ItemStyle>;
}

/** Static art drawn at its own size. */
export interface ImageArtProps extends ImagePlacement {
    art: ArtRef;
    bind?: never;
    size?: never;
    align?: never;
}

/** A runtime sprite slot the `sprite` handle `bind` fills. */
export interface ImageBindProps extends ImagePlacement {
    bind: SpriteHandle;
    art?: never;
    /** Slot size in pixels; a number is square. */
    size: number | [number, number];
    align?: TextAlign;
}

export type ImageProps = ImageArtProps | ImageBindProps;

/**
 * An inventory region: the slots it covers route clicks to `onClick` and show its hitbox item. Without `width` and
 * `height` it fills its parent box, or in a section its grid area.
 */
export interface RegionProps extends ItemProps, DebugProps {
    /** What a click does; omit for a hover-only region. */
    onClick?: ClickAction;
    tooltip?: string | Tooltip;
    /** Item model of the hitbox item filling its slots. */
    itemModel?: string;
    width?: number;
    height?: number;
    style?: StyleValue<ItemStyle>;
}

interface SlotSource {
    /** Raw slot refs; the control is then not placed on the section grid. */
    slots?: SlotList;
    /** A slot pattern; in a section it is relative to the control's grid area. */
    pattern?: SlotPatternInput;
}

interface CaseBoxProps extends Omit<BoxProps, keyof ItemProps | "x" | "y" | "style"> {
    style?: StyleValue<CaseStyle>;
}

/** A case box; its children lay out as a column by default. */
export interface CaseProps extends CaseBoxProps {
    /** Value the switch binding returns to draw this case. */
    value: string;
}

export interface SwitchProps extends ItemProps, DebugProps {
    /** What selects the case: a value or selection handle whose value names it, or a condition selecting `"true"`. */
    bind: Ref<"value" | "selection"> | Condition;
    x?: number;
    y?: number;
    text?: TextProps;
    style?: StyleValue<SwitchStyle>;
    children?: Child;
}

/** A switch on a value or selection handle, with one case per value. */
export interface SwitchOnProps<V extends string> extends ItemProps, DebugProps {
    on: Ref<"value" | "selection", V>;
    x?: number;
    y?: number;
    text?: TextProps;
    style?: StyleValue<SwitchStyle>;
    /** The content of each case, keyed by value; every value needs one. */
    children: { readonly [K in NoInfer<V>]: Child };
}

/** A switch on a flag or toggle handle, with a `true` and a `false` case. */
export interface SwitchFlagProps extends ItemProps, DebugProps {
    on: Flag | ToggleHandle;
    x?: number;
    y?: number;
    text?: TextProps;
    style?: StyleValue<SwitchStyle>;
    children: { readonly true: Child; readonly false: Child };
}

export interface ItemsProps extends ItemProps, SlotSource, DebugProps {
    /** The `items` handle rendering the stacks. */
    bind: ItemsHandle;
    style?: StyleValue<ItemStyle>;
}

export interface CollectionProps extends ItemProps, SlotSource, DebugProps {
    /** The `collection` handle supplying the cells. */
    bind: Collection;
    frame?: ArtRef;
    /** Art marking the selected cell. */
    selected?: ArtRef;
    /** `false` for a display-only collection. */
    action?: boolean;
    /** Width in slots; defaults to the full section width. */
    columns?: number;
    rows?: number;
    style?: StyleValue<CollectionStyle>;
}

export interface InputProps extends DebugProps {
    /** The `input` handle receiving the typed value. */
    bind: InputHandle;
    initial?: string;
    itemModel?: string;
}

export interface SectionProps extends DebugProps {
    frame?: ArtRef;
    /** How far the frame extends past the slot boxes. Defaults to 3 when a frame is set. */
    outset?: Insets;
    /** Claim for slots no child owns. Defaults to "unowned". */
    claim?: SlotRectClaim;
    flow?: GridFlow;
    text?: TextProps;
    style?: StyleValue<SectionStyle>;
    children?: Child;
}

/** A slot grid section: `of` names the container, the player's main inventory, or the hotbar. */
export interface SectionOfProps extends SectionProps {
    of: SlotSection;
}

export interface WindowProps extends DebugProps {
    name: string;
    container: ContainerKind;
    bleed?: Insets;
    frame?: ArtRef;
    text?: TextProps;
    style?: StyleValue<SectionStyle>;
    /** Overrides vars for the whole window. */
    theme?: VarTheme;
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

export interface HudProps extends Omit<BoxProps, keyof ItemProps | "x" | "y" | "frame" | "width" | "height" | "style"> {
    style?: StyleValue<HudStyle>;
    width?: number;
    height?: number;
    name: string;
    channel?: HudChannel;
    bleed?: Insets;
    frame?: ArtRef;
    /** Screen point the HUD is pinned to, through the generated core shaders. */
    anchor?: HudAnchor | { x: number; y: number };
    /** GUI-pixel nudge from the anchor. */
    offset?: [number, number];
    /** See HudShader.source_bottom. */
    sourceBottom?: number;
}

type RenderNode = Element | { windows: WindowDef[] } | { huds: HudDef[] } | CaseElement;

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
        if (!("type" in node) || node.type === "case") {
            throw new Error("Window/Hud roots belong in fragments, and <Case> inside <Switch>");
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

function requireName(name: unknown, component: string): void {
    if (typeof name !== "string" || name === "") {
        throw new Error(`<${component}> requires a non-empty \`name\``);
    }
}

/** The `handle` field of an element reading `ref`, which must be a handle of one of `kinds`. */
function bound(ref: unknown, kinds: readonly string[], label: string): Fields {
    if (!isRef(ref) || !kinds.includes(ref.kind)) {
        throw new Error(`${label} must be a ${kinds.map((kind) => `\`${kind}\``).join(" or ")} handle`);
    }
    return { handle: refJson(ref) };
}

/** A flexbox (or grid) container. Auto-sized boxes fill their parent's content box. */
export function Box(input: BoxProps): Element {
    const props = withStyle(input, STYLE_KEYS.box!, "Box");
    const node = clean({
        type: "flex",
        frame: props.frame,
        x: props.x,
        y: props.y,
        style: flexStyle(props as Fields),
        theme: props.theme,
        children: nodes(props.children),
        ...layout(props),
        ...debug(props),
    });
    return cascade(node as Element, props.text);
}

/** Static label text, or dynamic text with `bind`. */
export function Text(input: TextElementProps): Element {
    const props = withStyle(input, STYLE_KEYS.text!, "Text");
    const style = textFields(props as Fields);
    const common = clean({ width: props.width, x: props.x, y: props.y, ...style, ...layout(props), ...debug(props) });
    const fit = { overflow: props.overflow, lines: props.lines, line_height: props.lineHeight };
    if (props.bind !== undefined) {
        return clean({ type: "slot", ...bound(props.bind, ["text"], "<Text bind>"), ...common, ...fit }) as Element;
    }
    for (const [key, value] of [
        ["overflow", props.overflow],
        ["lines", props.lines],
        ["lineHeight", props.lineHeight],
    ]) {
        if (value !== undefined) {
            throw new Error(`<Text ${key}> requires \`bind\`; static labels do not fit their text at runtime`);
        }
    }
    const value = text(props.children);
    if (value === "") {
        throw new Error("<Text> requires text children or `bind`");
    }
    return { type: "label", text: value, ...common } as Element;
}

/** Inline art drawn at its own size, or with a `sprite` handle `bind` a runtime sprite slot. */
export function Image(input: ImageProps): Element {
    const props = withStyle(input, STYLE_KEYS.item!, "Image") as ImageProps;
    const placement = { x: props.x, y: props.y, ...layout(props), ...debug(props) };
    if (props.bind !== undefined) {
        const [width, height] = typeof props.size === "number" ? [props.size, props.size] : props.size;
        return clean({
            type: "sprite_slot",
            ...bound(props.bind, ["sprite"], "<Image bind>"),
            width,
            height,
            align: props.align,
            ...placement,
        }) as Element;
    }
    if (typeof props.art !== "object" || props.art === null) {
        throw new Error("<Image> requires `art`, such as texture(...) or shape(...), or a `sprite` handle `bind`");
    }
    return clean({ type: "sprite", art: props.art, ...placement }) as Element;
}

/**
 * An inventory region: the slots it covers take its clicks and show its hitbox item. It fills its parent box, or in
 * a section claims its `span`/`at` area; inside switch cases, regions claim their slots only while their case is drawn.
 */
export function Region(input: RegionProps): Element {
    const props = withStyle(input, STYLE_KEYS.item!, "Region");
    return region(
        clean({
            on_click: props.onClick,
            tooltip: props.tooltip,
            item_model: props.itemModel,
            width: props.width,
            height: props.height,
            ...layout(props),
            ...debug(props),
        }),
    ) as Element;
}

function sectionNode(kind: SlotSection, input: SectionProps, component: string): Element {
    const props = withStyle(input, STYLE_KEYS.section!, component);
    const node = clean({
        type: "section",
        section: kind,
        frame: props.frame,
        outset: props.outset ?? (props.frame === undefined ? undefined : 3),
        claim: props.claim,
        flow: props.flow,
        children: nodes(props.children),
        ...debug(props),
    });
    return cascade(node as Element, props.text);
}

/** An inventory slot grid whose children auto-flow through it, one track per slot. */
export function Section(props: SectionOfProps): Element {
    const { of, ...rest } = props;
    if (!["container", "player", "hotbar"].includes(of)) {
        throw new Error('<Section of> must be "container", "player", or "hotbar"');
    }
    return sectionNode(of, rest, "Section");
}

/** One case of a `<Switch>`. */
export function Case(props: CaseProps): CaseElement {
    requireName(props.value, "Case value");
    const { value, ...box } = props;
    const node = Box({ direction: "column", ...withStyle(box, STYLE_KEYS.case!, "Case") });
    if (node.type !== "flex") {
        throw new Error("<Case> must render a box");
    }
    return clean({
        type: "case",
        value,
        frame: node.frame,
        style: node.style,
        theme: node.theme,
        children: node.children ?? [],
        ...debug(props),
    }) as CaseElement;
}

/** A switch element drawing `cases`, selected by the handle JSON `binding`. */
function switchNode(
    props: ItemProps & DebugProps & { x?: number; y?: number; text?: TextProps },
    binding: Fields,
    cases: CaseElement[],
): Element {
    const node = clean({
        type: "switch",
        ...binding,
        x: props.x,
        y: props.y,
        children: cases,
        ...layout(props),
        ...debug(props),
    });
    return cascade(node as Element, props.text);
}

/**
 * Stacks its cases in one box sized to the largest case; the runtime draws only the active case. With `bind`, the
 * children are `<Case>` elements and the handle selects one; with `on`, the children map each value of a value or
 * selection handle, or `true` and `false` of a flag or toggle, to its content. Cases may hold regions, items, and
 * collections, which claim their slots only while their case is drawn.
 */
export function Switch<V extends string>(input: SwitchProps | SwitchOnProps<V> | SwitchFlagProps): Element {
    const props = withStyle(input, STYLE_KEYS.switch!, "Switch") as SwitchProps | SwitchOnProps<V> | SwitchFlagProps;
    if ("on" in props) {
        const content = props.children as Record<string, Child>;
        const boolean = props.on.kind === "flag" || props.on.kind === "toggle";
        const values: readonly string[] = boolean ? ["true", "false"] : (props.on.values ?? []);
        const extra = Object.keys(content).find((key) => !values.includes(key));
        if (extra !== undefined) {
            throw new Error(`<Switch on={${props.on.id}}> has a case \`${extra}\` that is not one of its values`);
        }
        const cases = values.map((value) => Case({ value, children: content[value] }));
        return switchNode(props, { handle: refJson(props.on) }, cases);
    }
    const cases = renderNodes(props.children).map((node) => {
        if (!("type" in node) || node.type !== "case") {
            throw new Error("<Switch> children must be <Case> elements");
        }
        return node;
    });
    return switchNode(props, bound(props.bind, ["value", "selection", "flag", "toggle"], "<Switch bind>"), cases);
}

/** Real item stacks in the slots it covers: its parent box's, or in a section its `span`/`at` area. */
export function Items(input: ItemsProps): Element {
    const props = withStyle(input, STYLE_KEYS.item!, "Items");
    return clean({
        type: "item",
        ...bound(props.bind, ["items"], "<Items bind>"),
        slots: props.slots,
        pattern: props.pattern,
        ...layout(props),
        ...debug(props),
    }) as Element;
}

/** A scrolling item collection; by default it spans the full section width. */
export function Collection(input: CollectionProps): Element {
    const props = withStyle(input, STYLE_KEYS.collection!, "Collection");
    const full =
        props.columns === undefined && props.span === undefined && props.at === undefined && props.col === undefined;
    return clean({
        type: "collection",
        ...bound(props.bind, ["collection"], "<Collection bind>"),
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
        ...debug(props),
    }) as Element;
}

/** The anvil's native rename field, bound as a text input. */
export function Input(props: InputProps): Element {
    return clean({
        type: "anvil_input",
        ...bound(props.bind, ["input"], "<Input bind>"),
        initial: props.initial,
        item_model: props.itemModel,
        ...debug(props),
    }) as Element;
}

/** A container window. Default-export it from a `.tsx` file under `window/`. */
export function Window(input: WindowProps): { windows: WindowDef[] } {
    const props = withStyle(input, STYLE_KEYS.section!, "Window");
    requireName(props.name, "Window");
    const def = clean({
        name: props.name,
        container: props.container,
        bleed: props.bleed,
        frame: props.frame,
        theme: props.theme,
        children: nodes(props.children),
        ...debug(props),
    }) as WindowDef;
    return ui(cascade(def, props.text));
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
export function Hud(input: HudProps): { huds: HudDef[] } {
    const props = withStyle(input, STYLE_KEYS.hud!, "Hud");
    requireName(props.name, "Hud");
    const { name, channel, bleed, frame, theme, anchor, offset, sourceBottom, width, height, debugName, ...box } =
        props;
    const fixed = typeof width === "number" && typeof height === "number";
    if (!fixed && (width !== undefined || height !== undefined)) {
        throw new Error("<Hud> `width` and `height` must both be pixel numbers, or both be omitted");
    }
    const named = debugName === undefined ? {} : { debugName };
    const root = Box({ direction: "column", ...named, ...box });
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
        theme,
        shader,
        width: fixed ? width : undefined,
        height: fixed ? height : undefined,
        children: [root],
        ...debug(named),
    }) as HudDef;
    return hud(def);
}
