import type { Child as JsxChild } from "#rpp/jsx";

import { hud, ui } from "./elements.ts";
import { builtin, isRef, refJson } from "./handles.ts";
import type { Action, ClickAction, Collection, Condition, Flag, Indexed, Input, Items, Ref, Shape } from "./handles.ts";
import type { Selection, Sprite as SpriteHandle, Text as TextHandle, Toggle as ToggleHandle } from "./handles.ts";
import type {
    AutoLength,
    BindingIndex,
    ButtonDefault,
    CaseElement,
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
    TextOverflow,
    Tooltip,
    Window as WindowDef,
} from "./types.ts";

export type Window = WindowDef;
export type Hud = HudDef;

type Fields = Record<string, unknown>;

/** Anything JSX can render as a child. Strings become labels; arrays and fragments flatten. */
export type Child = JsxChild;

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
    /** Dynamic text: a `text` handle, or a slot name. Without it, the children are static label text. */
    bind?: string | TextHandle;
    /** Places the bound slot in an indexed binding family. */
    index?: BindingIndex;
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

export interface SpriteProps extends ItemProps {
    name: string;
    x?: number;
    y?: number;
}

export interface IconProps extends ItemProps {
    /** Runtime sprite: a `sprite` handle, or a sprite slot name. */
    bind: string | SpriteHandle;
    /** Places the sprite slot in an indexed binding family. */
    index?: BindingIndex;
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

export interface ButtonProps<V extends string = string> extends ItemProps, SlotSource {
    /** Button name for a string binding; handle buttons are named by `onClick`. */
    name?: string;
    /** What a click does: an `action`, `selection.set(value)`, a `toggle` to flip, or a `builtin`. */
    onClick?: ClickAction;
    /** Ignores clicks and shows the `disabled` state while the condition is false. Requires `onClick`. */
    enabled?: Condition;
    /** State shown while `enabled` is false. */
    disabled?: StateProps;
    /** A value or selection whose current value names the state shown; `states` holds one per value. */
    state?: Ref<"value" | "selection", V>;
    frame?: string;
    tooltip?: string | Tooltip;
    states?: Record<NoInfer<V>, StateProps>;
    /** Close the window on click: `onClick={builtin("window:close")}`, or with `name` a closing default handler. */
    close?: boolean;
    padding?: number;
    gap?: FixedLength;
    text?: TextProps;
    children?: Child;
}

export interface ToggleProps extends Omit<ButtonProps, "states" | "state" | "enabled" | "disabled"> {
    /** The `toggle` handle a click flips; its value picks the `on` or `off` state. */
    bind?: ToggleHandle;
    on: StateProps;
    off: StateProps;
}

export interface ChoiceProps extends Omit<ButtonProps, "states" | "state" | "enabled" | "disabled"> {
    selected: StateProps;
    unselected: StateProps;
}

interface TabsStyle extends ItemProps {
    frame?: string;
    /** Sprite drawn behind an unselected tab. */
    sprite?: string;
    /** Sprite drawn behind the selected tab. */
    selectedSprite?: string;
    itemModel?: string;
    text?: TextProps;
}

export interface TabsProps extends TabsStyle {
    /** Prefix of every tab's choice name: `<name>_<value>`. */
    name: string;
    children?: Child;
}

/** One tab per value of `bind`, rendered by `children`; a click selects its value. */
export interface TabsBindProps<V extends string> extends Omit<TabsStyle, "itemModel"> {
    bind: Selection<V>;
    /** Each tab's tooltip; defaults to the text of its labels. */
    tooltip?: (value: V) => string | Tooltip;
    /** Every tab's item model, or each tab's. */
    itemModel?: string | ((value: V) => string);
    /** Renders the tab of `value`, the `index`th of the selection's values. */
    children: (value: V, index: number) => Child;
}

export interface TabProps {
    value: string;
    tooltip?: string | Tooltip;
    children?: Child;
}

type CaseBoxProps = Omit<BoxProps, keyof ItemProps | "x" | "y">;

/** A case box; its children lay out as a column by default. */
export interface CaseProps extends CaseBoxProps {
    /** Value the switch binding returns to draw this case. */
    value: string;
}

export interface SwitchProps extends ItemProps {
    /** Binding name of the active case value. */
    bind: string;
    /** Places the switch in an indexed binding family; every entry must have the same case values. */
    index?: BindingIndex;
    x?: number;
    y?: number;
    text?: TextProps;
    children?: Child;
}

/** A switch on a value or selection handle, with one case per value. */
export interface SwitchOnProps<V extends string> extends ItemProps {
    on: Ref<"value" | "selection", V>;
    x?: number;
    y?: number;
    text?: TextProps;
    /** The content of each case, keyed by value; every value needs one. */
    children: { readonly [K in NoInfer<V>]: Child };
}

/** A switch on a flag or toggle handle, with a `true` and a `false` case. */
export interface SwitchFlagProps extends ItemProps {
    on: Flag | ToggleHandle;
    x?: number;
    y?: number;
    text?: TextProps;
    children: { readonly true: Child; readonly false: Child };
}

/** Box props apply to the shown case; item props place the switch. */
export interface ShowProps extends ItemProps, CaseBoxProps {
    /** A condition handle, or a Boolean binding name. */
    when: string | Condition;
    /** Places the switch in an indexed binding family. */
    index?: BindingIndex;
    x?: number;
    y?: number;
}

export interface HotspotProps extends ItemProps, SlotSource {
    name: string;
    tooltip?: string | Tooltip;
    states?: Record<string, StateProps>;
}

export interface ItemSlotProps extends ItemProps, SlotSource {
    /** Item name for a string binding; set either `name` or `bind`. */
    name?: string;
    /** The `items` handle rendering this item. */
    bind?: Items;
    /** One-based slot inside the enclosing repeater cell. */
    cellSlot?: number;
}

export interface CollectionProps extends ItemProps, SlotSource {
    /** Collection name for a string binding; set either `name` or `bind`. */
    name?: string;
    /** The `collection` handle supplying the cells. */
    bind?: Collection;
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
    /** Repeater name; a repeater rendering its cells with a function may use `onClick` instead. */
    name?: string;
    /** An action indexed by cell, `shape: [cells]`; requires children rendered by a function. */
    onClick?: Indexed<Action, Shape>;
    /** Size of one cell in slots. */
    cell: [number, number];
    columns: number;
    rows: number;
    frame?: string;
    padding?: number;
    text?: TextProps;
    /** A template repeated in every cell, or a function rendering cell `i` that reads indexed handles with `.at(i)`. */
    children?: Child | ((i: number) => Child);
}

export interface SlotsProps extends ItemProps {
    name: string;
    frame?: string;
    claim?: SlotRectClaim;
}

export interface AnvilInputProps {
    /** Input name for a string binding; set either `name` or `bind`. */
    name?: string;
    /** The `input` handle receiving the typed value. */
    bind?: Input;
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

type RenderNode = Element | { windows: WindowDef[] } | { huds: HudDef[] } | TabNode | CaseElement;

interface TabNode {
    type: "tab";
    value: string;
    tooltip: string | Tooltip | undefined;
    children: Element[];
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
        if (!("type" in node) || node.type === "tab" || node.type === "case") {
            throw new Error(
                "Window/Hud roots belong in fragments; <Tab> belongs inside <Tabs> and <Case> inside <Switch>",
            );
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
        if (Array.isArray(el.cells)) {
            result.cells = (el.cells as Fields[][]).map((cell) => cell.map(visit));
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

const ITEM_KEYS: readonly string[] = [
    "grow",
    "shrink",
    "basis",
    "alignSelf",
    "justifySelf",
    "margin",
    "absolute",
    "top",
    "right",
    "bottom",
    "left",
    "span",
    "at",
    "col",
    "row",
    "translate",
] satisfies (keyof ItemProps)[];

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

/** The name or handle an element binds: `{ name }` for a string, `{ handle }` for a handle reference. */
function binding(value: unknown, component: string): Fields {
    if (isRef(value)) {
        return { handle: refJson(value) };
    }
    requireName(value, component);
    return { name: value };
}

/** One of `name` or `bind`, which handle-capable controls take instead of a required `name`. */
function nameOrBind(props: { name?: string | undefined; bind?: Ref | undefined }, component: string): Fields {
    if ((props.name === undefined) === (props.bind === undefined)) {
        throw new Error(`<${component}> requires exactly one of \`name\` or \`bind\``);
    }
    return binding(props.name ?? props.bind, component);
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
    const fit = { overflow: props.overflow, lines: props.lines, line_height: props.lineHeight };
    if (props.bind !== undefined) {
        return clean({
            type: "slot",
            ...binding(props.bind, "Text bind"),
            index: props.index,
            ...common,
            ...fit,
        }) as Element;
    }
    if (props.index !== undefined) {
        throw new Error("<Text index> requires `bind`");
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

/** A static theme sprite. */
export function Sprite(props: SpriteProps): Element {
    requireName(props.name, "Sprite");
    return clean({ type: "sprite", name: props.name, x: props.x, y: props.y, ...layout(props) }) as Element;
}

/** A runtime sprite slot, drawn above button and selection backgrounds. */
export function Icon(props: IconProps): Element {
    const [width, height] = typeof props.size === "number" ? [props.size, props.size] : props.size;
    return clean({
        type: "sprite_slot",
        ...binding(props.bind, "Icon bind"),
        index: props.index,
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

/** The name or click handle, `enabled`, and `state` fields of a button. */
function buttonBinding(props: ButtonProps): Fields {
    const { name, onClick, enabled, disabled, state: value } = props;
    if (onClick === undefined && name === undefined && props.close !== true) {
        throw new Error("<Button> requires `name` or `onClick`");
    }
    if (onClick !== undefined && (name !== undefined || props.close === true)) {
        throw new Error(
            '<Button> sets `onClick` with `name` or `close`; `close` is `onClick={builtin("window:close")}`',
        );
    }
    if (name !== undefined && (enabled !== undefined || value !== undefined)) {
        throw new Error("<Button enabled> and <Button state> require `onClick`");
    }
    if (disabled !== undefined && enabled === undefined) {
        throw new Error("<Button disabled> requires `enabled`");
    }
    if (enabled !== undefined && props.states !== undefined) {
        throw new Error("<Button enabled> draws its `disabled` state; it cannot also set `states`");
    }
    if (name !== undefined) {
        requireName(name, "Button");
        return { name, default: props.close === true ? ("close" satisfies ButtonDefault) : undefined };
    }
    return {
        on_click: refJson(onClick ?? builtin("window:close")),
        enabled: enabled === undefined ? undefined : refJson(enabled),
        state: value === undefined ? undefined : refJson(value),
        ...(enabled === undefined ? {} : { states: { enabled: {}, disabled: state(disabled ?? {}) } }),
    };
}

/** Content children are centered in a row filling the button. */
export function Button<V extends string = string>(props: ButtonProps<V>): Element {
    const bound = buttonBinding(props as ButtonProps);
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
        frame: props.frame,
        tooltip: props.tooltip,
        states: states(props.states),
        ...bound,
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
    const { on, off, bind, ...rest } = props;
    if (bind !== undefined && rest.onClick !== undefined) {
        throw new Error("<Toggle> sets both `bind` and `onClick`");
    }
    return Button({ ...rest, ...(bind === undefined ? {} : { onClick: bind }), states: { on, off } });
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

/** A tab's tooltip: its own, or the text of its labels. */
function tabTooltip(tooltip: string | Tooltip | undefined, children: Element[]): string | Tooltip | undefined {
    const label = tooltip ?? children.flatMap((c) => (c.type === "label" ? [c.text] : [])).join(" ");
    return label === "" ? undefined : label;
}

function tab(props: TabsStyle, choice: Fields, tooltip: string | Tooltip | undefined, children: Element[]): Element {
    const { frame, sprite, selectedSprite, itemModel, text, ...item } = props;
    const base = clean({ itemModel, tooltip: tabTooltip(tooltip, children) });
    return Choice({
        ...item,
        ...choice,
        ...clean({ frame, text }),
        selected: clean({ ...base, sprite: selectedSprite }),
        unselected: clean({ ...base, sprite }),
        children,
    });
}

/**
 * A choice group. With `name`, each `<Tab value>` becomes the choice `<name>_<value>`; with a selection `bind`, the
 * render function draws one tab per value and a click selects it.
 */
export function Tabs<V extends string>(props: TabsProps | TabsBindProps<V>): Element[] {
    if ("bind" in props) {
        const { bind, children, tooltip, itemModel, ...style } = props;
        return bind.values.map((value, i) => {
            const model = typeof itemModel === "function" ? itemModel(value) : itemModel;
            const choice = { onClick: bind.set(value) };
            return tab(
                { ...style, ...clean({ itemModel: model }) },
                choice,
                tooltip?.(value),
                nodes(children(value, i)),
            );
        });
    }
    requireName(props.name, "Tabs");
    const { name, children, ...style } = props;
    return renderNodes(children).map((t) => {
        if (!("type" in t) || t.type !== "tab") {
            throw new Error("<Tabs> children must be <Tab> elements");
        }
        return tab(style, { name: `${name}_${t.value}` }, t.tooltip, t.children);
    });
}

/** One case of a `<Switch>`. */
export function Case(props: CaseProps): CaseElement {
    requireName(props.value, "Case value");
    const { value, ...box } = props;
    const node = Box({ direction: "column", ...box });
    if (node.type !== "flex") {
        throw new Error("<Case> must render a box");
    }
    return clean({
        type: "case",
        value,
        frame: node.frame,
        style: node.style,
        children: node.children ?? [],
    }) as CaseElement;
}

/** A switch element drawing `cases`, bound by `binding`. */
function switchNode(
    props: ItemProps & { x?: number; y?: number; index?: BindingIndex; text?: TextProps },
    binding: Fields,
    cases: CaseElement[],
): Element {
    const node = clean({
        type: "switch",
        ...binding,
        index: props.index,
        x: props.x,
        y: props.y,
        children: cases,
        ...layout(props),
    });
    return cascade(node as Element, props.text);
}

/**
 * Stacks its cases in one box sized to the largest case; the runtime draws only the active case. With `bind`, the
 * children are `<Case>` elements and the binding names the active one; with `on`, the children map each value of a
 * value or selection handle, or `true` and `false` of a flag or toggle, to its content. Cases are visual: art, text,
 * and icons, but no slot-bound controls.
 */
export function Switch<V extends string>(props: SwitchProps | SwitchOnProps<V> | SwitchFlagProps): Element {
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
    requireName(props.bind, "Switch bind");
    const cases = renderNodes(props.children).map((node) => {
        if (!("type" in node) || node.type !== "case") {
            throw new Error("<Switch> children must be <Case> elements");
        }
        return node;
    });
    return switchNode(props, { name: props.bind }, cases);
}

/** Draws its children only while the condition `when` is true; their space is always reserved. */
export function Show(props: ShowProps): Element {
    const box = Object.fromEntries(
        Object.entries(props).filter(([key]) => !ITEM_KEYS.includes(key) && !["when", "index", "x", "y"].includes(key)),
    ) as CaseBoxProps;
    const { text: _text, children: _children, ...placement } = props;
    return switchNode(placement, binding(props.when, "Show when"), [
        Case({ ...box, value: "true" }),
        Case({ value: "false" }),
    ]);
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
    return clean({
        type: "item",
        ...nameOrBind(props, "Item"),
        cell_slot: props.cellSlot,
        slots: props.slots,
        pattern: props.pattern,
        ...layout(props),
    }) as Element;
}

/** A scrolling item collection; by default it spans the full section width. */
export function Collection(props: CollectionProps): Element {
    const bound = nameOrBind(props, "Collection");
    const full =
        props.columns === undefined && props.span === undefined && props.at === undefined && props.col === undefined;
    return clean({
        type: "collection",
        ...bound,
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

/** One cell's children: `<Item cellSlot>` children stay in the cell; the rest are centered in a column filling it. */
function cell(children: Child): Element[] {
    const all = nodes(children);
    const kids = all.filter((c) => c.type !== "item");
    return [
        ...all.filter((c) => c.type === "item"),
        ...(kids.length === 0 ? [] : [Column({ justify: "center", align: "center", children: kids })]),
    ];
}

/**
 * A grid of `columns` x `rows` cells, each a button of `cell` slots. Children are a template repeated in every cell,
 * or a function rendering cell `i` (row-major) that reads indexed handles with `.at(i)`; `onClick` then receives
 * the clicked cell's index.
 */
export function Repeater(props: RepeaterProps): Element {
    const { children, onClick } = props;
    const rendered = typeof children === "function";
    if (!rendered && (onClick !== undefined || props.name === undefined)) {
        throw new Error(
            "<Repeater> with template children requires `name` and no `onClick`; render cells with a function",
        );
    }
    if (props.name !== undefined) {
        requireName(props.name, "Repeater");
    }
    const count = props.columns * props.rows;
    const node = clean({
        type: "repeater",
        name: props.name,
        on_click: onClick === undefined ? undefined : refJson(onClick),
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
        children: rendered ? undefined : cell(children),
        cells: rendered ? Array.from({ length: count }, (_, i) => cell(children(i))) : undefined,
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
    return clean({
        type: "anvil_input",
        ...nameOrBind(props, "AnvilInput"),
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
