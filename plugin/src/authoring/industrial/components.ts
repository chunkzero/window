/**
 * Industrial's components, built from the public primitives. Each takes a `style` of the variants it names and
 * defaults to industrial's styles, so `style` is only for overrides.
 */
import { Box, Case, Collection as CollectionPrimitive, Items, Region, Section, Switch } from "../jsx.ts";
import type { Child, CollectionProps as CollectionPrimitiveProps, DebugProps, SectionProps } from "../jsx.ts";
import { builtin } from "../handles.ts";
import { assign } from "../styles.ts";
import type { Action, ClickAction, Condition, Indexed, Items as ItemsHandle, Selection, Toggle } from "../handles.ts";
import type {
    BoxStyle,
    CaseStyle,
    ContainerStyle,
    ItemStyle,
    StyleFor,
    StyleValue,
    TextProps,
    Variants,
} from "../styles.ts";
import type { ArtRef, Element, FixedLength, Tooltip } from "../types.ts";
import { styles } from "./styles.ts";

type Layer = StyleValue<ContainerStyle>;

/** A `style` prop taking some of the variants `K`, each a box container style. */
export type VariantStyle<K extends string> = Partial<Variants<K, ContainerStyle>>;

/** What one state of a control shows and does. */
export interface StateProps {
    tooltip?: string | Tooltip;
    /** Item model of the hitbox item filling the control's slots in this state. */
    itemModel?: string;
    /** Frame drawn at the control's size in this state. */
    frame?: ArtRef;
}

/** A control's face: its frame, content layout, and the text style of its labels. */
interface FaceProps {
    /** Frame of the face; a state's `frame` replaces it. */
    frame?: ArtRef;
    tooltip?: string | Tooltip;
    itemModel?: string;
    /** Inset of the content from the face's edges. */
    padding?: number;
    /** Space between content children; defaults to 2. */
    gap?: FixedLength;
    /** Text style of the labels inside; labels are centered by default. */
    text?: TextProps;
    children?: Child;
}

/** Placement of a control: in a section, `span` and `at` pick its slots. */
interface ControlProps extends ItemStyle, DebugProps, FaceProps {}

const FACE: StyleFor<ContainerStyle> = { direction: "row", justify: "center", align: "center", gap: 2 };

function flat(value: unknown, out: Record<string, unknown>[] = []): Record<string, unknown>[] {
    if (Array.isArray(value)) {
        value.forEach((entry) => flat(entry, out));
    } else if (typeof value === "object" && value !== null) {
        out.push(value as Record<string, unknown>);
    }
    return out;
}

/** `fields` without its undefined entries. */
function defined<T extends object>(fields: T): { [K in keyof T]?: Exclude<T[K], undefined> } {
    return Object.fromEntries(Object.entries(fields).filter(([, value]) => value !== undefined)) as never;
}

/**
 * The style and text of a face: `base` layers, then the explicit props, then the `state` layers and the state's
 * frame. Labels are centered unless a layer or `text` aligns them.
 */
function face(base: readonly Layer[], state: readonly Layer[], props: FaceProps, current: StateProps = {}) {
    const explicit = defined({ frame: props.frame, padding: props.padding, gap: props.gap });
    const style: Layer[] = [FACE, ...base, explicit, ...state, defined({ frame: current.frame })];
    const text: Record<string, unknown> = { align: "center" };
    for (const fields of [...flat(style).map((entry) => entry["text"]), props.text]) {
        if (typeof fields === "object" && fields !== null) {
            assign(text, fields as Record<string, unknown>);
        }
    }
    return { style, text: text as TextProps };
}

/** The props of `props` that place the control rather than draw it. */
function placement(props: object, drawn: readonly string[]): ItemStyle & DebugProps {
    return Object.fromEntries(Object.entries(props).filter(([key]) => !drawn.includes(key))) as ItemStyle & DebugProps;
}

const FACE_KEYS = ["frame", "tooltip", "itemModel", "padding", "gap", "text", "children", "style"];

/** A control's box: in a section, its slot cells inset by 1px, so its face covers the slot interiors. */
function control(props: object, drawn: readonly string[], content: Child): Element {
    return Box({ ...placement(props, [...FACE_KEYS, ...drawn]), padding: 1, children: content });
}

/** The region of a face: it takes the face's clicks and shows its tooltip. */
function region(props: FaceProps, current: StateProps, onClick: ClickAction | undefined): Element {
    const tooltip = current.tooltip ?? props.tooltip;
    const itemModel = current.itemModel ?? props.itemModel;
    return Region(defined({ onClick, tooltip, itemModel }));
}

/** A face without states, filling its control. */
function plain(base: readonly Layer[], props: FaceProps, onClick: ClickAction): Element {
    const { style, text } = face(base, [], props);
    const children = [props.children, region(props, {}, onClick)];
    return Box({ grow: 1, style: style as StyleValue<BoxStyle>, text, children });
}

/** The face of one state, as the `value` case of its control's switch. */
function state(
    value: string,
    layers: [readonly Layer[], readonly Layer[]],
    props: FaceProps,
    current: StateProps,
    onClick?: ClickAction,
): Child {
    const { style, text } = face(layers[0], layers[1], props, current);
    const children = [props.children, region(props, current, onClick)];
    return Case({ value, style: style as StyleValue<CaseStyle>, text, children });
}

/** A control showing its `true` face while `when` holds and its `false` face otherwise. */
function switched(props: object, drawn: readonly string[], when: Condition, faces: [Child, Child]): Element {
    return control(props, drawn, Switch({ bind: when, grow: 1, children: faces }));
}

export interface ButtonProps extends ControlProps {
    /** What a click does: an `action`, `selection.set(value)`, a `toggle` to flip, or a `builtin`. */
    onClick?: ClickAction;
    /** Closes the window on click; the same as `onClick={builtin("window:close")}`. */
    close?: boolean;
    /** Shows the `disabled` state, which takes no clicks, while the condition is false. */
    enabled?: Condition;
    /** What the button shows while `enabled` is false. */
    disabled?: StateProps;
    /** `base` styles the face; `disabled` adds to it while `enabled` is false. */
    style?: VariantStyle<"base" | "disabled">;
}

/** A clickable face over its slots, its content centered in a row. */
export function Button(props: ButtonProps): Element {
    if ((props.onClick === undefined) === (props.close !== true)) {
        throw new Error("<industrial.Button> requires exactly one of `onClick` or `close`");
    }
    if (props.disabled !== undefined && props.enabled === undefined) {
        throw new Error("<industrial.Button disabled> requires `enabled`");
    }
    const onClick = props.onClick ?? builtin("window:close");
    const drawn = ["onClick", "close", "enabled", "disabled"];
    const base = [styles.button.base, props.style?.base];
    if (props.enabled === undefined) {
        return control(props, drawn, plain(base, props, onClick));
    }
    const disabled = [styles.button.disabled, props.style?.disabled];
    return switched(props, drawn, props.enabled, [
        state("true", [base, []], props, {}, onClick),
        state("false", [base, disabled], props, props.disabled ?? {}),
    ]);
}

export interface ToggleProps extends ControlProps {
    /** The `toggle` handle a click flips; its value picks the `on` or `off` state. */
    bind: Toggle;
    on?: StateProps;
    off?: StateProps;
    /** `base` styles the face; `on` and `off` add to it in their state. */
    style?: VariantStyle<"base" | "on" | "off">;
}

/** A button that flips a `toggle` handle and shows its `on` or `off` state. */
export function Toggle(props: ToggleProps): Element {
    const base = [styles.toggle.base, props.style?.base];
    return switched(props, ["bind", "on", "off"], props.bind, [
        state("true", [base, [props.style?.on]], props, props.on ?? {}, props.bind),
        state("false", [base, [props.style?.off]], props, props.off ?? {}, props.bind),
    ]);
}

export interface ChoiceProps<V extends string> extends ControlProps {
    /** The selection a click sets to `value`. */
    bind: Selection<V>;
    value: NoInfer<V>;
    selected?: StateProps;
    unselected?: StateProps;
    /** `base` styles the face; `selected` and `unselected` add to it in their state. */
    style?: VariantStyle<"base" | "selected" | "unselected">;
}

/** A button that selects one value of a selection, showing whether it is the selected one. */
export function Choice<V extends string>(props: ChoiceProps<V>): Element {
    const base = [styles.choice.base, props.style?.base];
    const selected = [styles.choice.selected, props.style?.selected];
    const onClick = props.bind.set(props.value);
    return switched(props, ["bind", "value", "selected", "unselected"], props.bind.is(props.value), [
        state("true", [base, selected], props, props.selected ?? {}, onClick),
        state("false", [base, [props.style?.unselected]], props, props.unselected ?? {}, onClick),
    ]);
}

export interface TabProps {
    value: string;
    tooltip?: string | Tooltip;
    children?: Child;
}

interface TabNode {
    readonly tab: TabProps;
}

/** One tab of a `<Tabs>` group, for tabs written out instead of rendered per value. */
export function Tab(props: TabProps): Child {
    return { tab: props } satisfies TabNode as Child;
}

export interface TabsProps<V extends string> extends Omit<ControlProps, "tooltip" | "itemModel" | "children"> {
    bind: Selection<V>;
    /** Art drawn at an unselected tab's size, instead of `frame`. */
    sprite?: ArtRef;
    /** Art drawn at the selected tab's size. */
    selectedSprite?: ArtRef;
    /** Each tab's tooltip; defaults to the text of its labels. */
    tooltip?: (value: V) => string | Tooltip;
    /** Every tab's item model, or each tab's. */
    itemModel?: string | ((value: V) => string);
    /** `base` styles every tab; `selected` and `unselected` add to it in their state. */
    style?: VariantStyle<"base" | "selected" | "unselected">;
    /** Renders the tab of `value`, the `index`th of the selection's values; or `<Tab>` elements, one per value. */
    children: ((value: V, index: number) => Child) | Child;
}

/** A tab's tooltip: its own, or the text of its top-level labels and plain text. */
function labelText(children: Child): string | undefined {
    const parts: string[] = [];
    const visit = (child: unknown): void => {
        if (Array.isArray(child)) {
            child.forEach(visit);
        } else if (typeof child === "string" || typeof child === "number") {
            parts.push(String(child).trim());
        } else if (typeof child === "object" && child !== null && (child as Element)["type"] === "label") {
            parts.push((child as { text: string }).text);
        }
    };
    visit(children);
    const text = parts.filter((part) => part !== "").join(" ");
    return text === "" ? undefined : text;
}

/** One choice per value of `bind`; a click selects its value. Each tab takes the item props, such as `span`. */
export function Tabs<V extends string>(props: TabsProps<V>): Element[] {
    const { bind, children, sprite, selectedSprite, tooltip, itemModel, ...rest } = props;
    const tabs = new Map<string, TabProps>();
    if (typeof children !== "function") {
        for (const node of flat(children)) {
            const tab = (node as Partial<TabNode>).tab;
            if (tab === undefined || !bind.values.includes(tab.value as V)) {
                throw new Error(`<industrial.Tabs> children must be <Tab> elements with values of \`${bind.id}\``);
            }
            tabs.set(tab.value, tab);
        }
    }
    return bind.values.map((value, i) => {
        const tab = tabs.get(value);
        const content = typeof children === "function" ? children(value, i) : tab?.children;
        const tip = tab?.tooltip ?? tooltip?.(value) ?? labelText(content);
        const model = typeof itemModel === "function" ? itemModel(value) : itemModel;
        const state = (frame: ArtRef | undefined): StateProps => defined({ frame });
        return Choice({
            ...rest,
            ...defined({ tooltip: tip, itemModel: model }),
            bind,
            value,
            selected: state(selectedSprite),
            unselected: state(sprite),
            children: content,
        });
    });
}

export interface HotspotProps extends ItemStyle, DebugProps {
    tooltip: string | Tooltip;
    itemModel?: string;
}

/** A tooltip over its slots; it takes no clicks. */
export function Hotspot(props: HotspotProps): Element {
    return Region(props);
}

export interface CollectionProps extends Omit<CollectionPrimitiveProps, "name" | "bind" | "frame" | "selected"> {
    bind: NonNullable<CollectionPrimitiveProps["bind"]>;
    /** Frame of every cell; defaults to industrial's slot. */
    frame?: ArtRef;
    /** Art over the selected cell; defaults to industrial's selected slot. */
    selected?: ArtRef;
}

/** A scrolling item collection drawn as industrial slots; by default it spans the full section width. */
export function Collection(props: CollectionProps): Element {
    return CollectionPrimitive({
        selected: styles.collection.selected,
        ...props,
        style: [{ frame: styles.collection.frame }, props.style],
    });
}

/** The opened container's slot grid. */
export function Container(props: SectionProps): Element {
    return Section({ ...props, of: "container" });
}

/** The player's 9x3 main inventory grid. */
export function Player(props: SectionProps): Element {
    return Section({ ...props, of: "player" });
}

/** The player's 9x1 hotbar grid. */
export function Hotbar(props: SectionProps): Element {
    return Section({ ...props, of: "hotbar" });
}

type BoxProps = Parameters<typeof Box>[0];

/** A horizontal box whose children are vertically centered by default. */
export function Row(props: BoxProps): Element {
    return Box({ ...props, style: [{ align: "center" }, props.style], direction: "row" });
}

export function Column(props: BoxProps): Element {
    return Box({ ...props, direction: "column" });
}

export function Grid(props: BoxProps): Element {
    return Box({ ...props, display: "grid" });
}

/** A box that centers its children on both axes. */
export function Center(props: BoxProps): Element {
    return Box({ ...props, style: [{ justify: "center", align: "center" }, props.style] });
}

/** Flexible empty space; in a section it skips `span` slots. */
export function Spacer(props: ItemStyle & DebugProps): Element {
    return Box({ grow: 1, ...props });
}

/** The title strip above the container grid, centering its children. */
export function Header(props: BoxProps): Element {
    return Box({
        ...props,
        style: [{ justify: "center", align: "center", gap: 4 }, props.style],
        x: 0,
        y: 0,
        width: 176,
        height: 17,
    });
}

export interface ShowProps extends ItemStyle, DebugProps, Omit<CaseStyle, "width" | "height"> {
    when: Condition;
    frame?: ArtRef;
    children?: Child;
}

/** Draws its children only while `when` holds; their space is always reserved. */
export function Show(props: ShowProps): Element {
    const { when, children, frame, debugName: _, ...rest } = props;
    const item: Record<string, unknown> = {};
    const box: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(rest)) {
        (ITEM_KEYS.has(key) ? item : box)[key] = value;
    }
    return Switch({
        ...(item as ItemStyle),
        ...defined({ debugName: props.debugName }),
        bind: when,
        children: [Case({ ...box, ...defined({ frame }), value: "true", children }), Case({ value: "false" })],
    });
}

const ITEM_KEYS: ReadonlySet<string> = new Set([
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
]);

export interface SlotsProps extends ItemStyle, DebugProps {
    /** Frame of every slot's 18x18 box; defaults to industrial's slot. */
    frame?: ArtRef;
    /** `"all"` claims the slots, so they take no clicks; defaults to `"none"`. */
    claim?: "none" | "all";
}

/** Draws a frame on each of its `span` slots. */
export function Slots(props: SlotsProps): Element {
    const { frame = styles.slots.frame, claim = "none", span = 1, ...item } = props;
    const [columns, rows] = typeof span === "number" ? [span, 1] : span;
    const cells = Array.from({ length: columns * rows }, () => Box({ frame }));
    return Box({
        ...item,
        span,
        display: "grid",
        columns,
        rows,
        children: [cells, claim === "all" ? Region({}) : null],
    });
}

export interface RepeaterProps extends ItemStyle, DebugProps {
    /** Size of one cell in slots. */
    cell: [number, number];
    columns: number;
    rows: number;
    /** An action indexed by cell; a click calls it with the clicked cell's index. */
    onClick?: Indexed<Action, readonly [number]>;
    tooltip?: string | Tooltip | ((i: number) => string | Tooltip);
    /** Items indexed by cell, shown in each cell's `itemSlot`, which then takes no clicks. */
    item?: Indexed<ItemsHandle, readonly [number]>;
    /** One-based slot of the cell `item` fills; defaults to 1. */
    itemSlot?: number;
    frame?: ArtRef;
    padding?: number;
    text?: TextProps;
    /** `cell` styles every cell's face. */
    style?: VariantStyle<"cell">;
    /** Content repeated in every cell, or a function rendering cell `i` (row-major) with indexed handles. */
    children?: Child | ((i: number) => Child);
}

const SLOT = 18;

/** A grid of `columns` x `rows` cells, each a face over `cell` slots whose content is centered in a column. */
export function Repeater(props: RepeaterProps): Element {
    const { cell, columns, rows, onClick, tooltip, item, itemSlot = 1, frame, padding, text, style, children } = props;
    const [width, height] = cell;
    if (item !== undefined && (itemSlot < 1 || itemSlot > width * height)) {
        throw new Error(`<industrial.Repeater itemSlot> must be between 1 and ${width * height}`);
    }
    const placed = placement(props, [
        "cell",
        "columns",
        "rows",
        "onClick",
        "tooltip",
        "item",
        "itemSlot",
        "frame",
        "padding",
        "text",
        "style",
        "children",
    ]);
    const layers: Layer[] = [
        { direction: "column", justify: "center", align: "center" },
        styles.repeater.cell,
        style?.cell,
        defined({ frame, padding }),
    ];
    const cells = Array.from({ length: columns * rows }, (_, i) => {
        const tip = typeof tooltip === "function" ? tooltip(i) : tooltip;
        const click = onClick?.at(i);
        const regions = (rects: [number, number, number][]) =>
            rects.map(([x, y, slots]) =>
                Region({
                    ...defined({ onClick: click, tooltip: tip }),
                    absolute: true,
                    left: x * SLOT,
                    top: y * SLOT,
                    width: slots * SLOT - 2,
                    height: SLOT - 2,
                }),
            );
        let claims: Child = Region(defined({ onClick: click, tooltip: tip }));
        let stack: Child = null;
        if (item !== undefined) {
            const [x, y] = [(itemSlot - 1) % width, Math.floor((itemSlot - 1) / width)];
            const rects: [number, number, number][] = [];
            for (let row = 0; row < height; row++) {
                if (row !== y) {
                    rects.push([0, row, width]);
                    continue;
                }
                if (x > 0) {
                    rects.push([0, row, x]);
                }
                if (x < width - 1) {
                    rects.push([x + 1, row, width - x - 1]);
                }
            }
            claims = regions(rects);
            const items = Items({ bind: item.at(i) });
            stack = Box({
                absolute: true,
                left: x * SLOT,
                top: y * SLOT,
                width: SLOT - 2,
                height: SLOT - 2,
                children: items,
            });
        }
        const content = typeof children === "function" ? children(i) : children;
        const box = Box({
            grow: 1,
            style: layers as StyleValue<BoxStyle>,
            ...defined({ text }),
            children: [content, stack, claims],
        });
        return Box({ padding: 1, children: box });
    });
    return Box({
        span: [columns * width, rows * height],
        ...placed,
        display: "grid",
        columns,
        rows,
        children: cells,
    });
}
