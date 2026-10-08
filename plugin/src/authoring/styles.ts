/** Build-time styles: `create`, `variants`, and the style properties each primitive accepts. See docs/AUTHORING.md. */
import type { Color, Var } from "./tokens.ts";
import type {
    ArtRef,
    AutoLength,
    Edges,
    FixedLength,
    FlexAlign,
    FlexJustify,
    FlexStyle,
    GridFlow,
    GridLine,
    ItemLayout,
    Length,
    TextAlign,
} from "./types.ts";

type Fields = Record<string, unknown>;

/** Text styling. Containers pass it down through `text`; the nearest setting wins. */
export interface TextProps {
    color?: string | Var<Color>;
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
export interface ItemStyle {
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

/** A box's frame, CSS container properties, and the text style its descendants inherit. */
export interface ContainerStyle {
    /** A theme frame name or inline art, stretched over the box's laid-out size. */
    frame?: ArtRef;
    display?: "flex" | "grid";
    direction?: "row" | "column" | "row-reverse" | "column-reverse";
    wrap?: boolean;
    justify?: FlexJustify;
    align?: FlexAlign;
    alignContent?: FlexJustify;
    gap?: FixedLength | [FixedLength, FixedLength] | Var<number>;
    padding?: Edges<FixedLength> | Var<number>;
    width?: Length;
    height?: Length;
    minWidth?: AutoLength;
    minHeight?: AutoLength;
    maxWidth?: AutoLength;
    maxHeight?: AutoLength;
    aspectRatio?: number;
    /** Grid columns: a count of equal `1fr` tracks, or track sizes such as `24`, `"1fr"`, or `"auto"`. */
    columns?: FlexStyle["columns"];
    rows?: FlexStyle["rows"];
    flow?: GridFlow;
    /** Default text style of the text inside. */
    text?: TextProps;
}

/** Style properties of `<Box>`. */
export interface BoxStyle extends ItemStyle, ContainerStyle {}
/** Style properties of `<Text>`. */
export interface TextStyle extends ItemStyle, TextProps {}
/** Style properties of `<Switch>`. */
export interface SwitchStyle extends ItemStyle {
    text?: TextProps;
}
/** Style properties of `<Collection>`. */
export interface CollectionStyle extends ItemStyle {
    frame?: ArtRef;
}
/** Style properties of `<Section>`, `<Window>`, and the raw `ui` and `hud`. */
export interface SectionStyle {
    frame?: ArtRef;
    text?: TextProps;
}
/** Style properties of `<Case>`. */
export type CaseStyle = ContainerStyle;
/** Style properties of `<Hud>`, whose children lay out in its root box. */
export type HudStyle = Omit<ContainerStyle, "width" | "height">;

/** Every style property; `create` accepts these. `align` aligns box children, or text within its width. */
export interface Style extends ItemStyle, Omit<ContainerStyle, "align">, Omit<TextProps, "align"> {
    align?: FlexAlign | TextAlign;
}

export const VARIANTS: unique symbol = Symbol("window.variants");

/** A style a primitive accepting `S` takes: properties outside `S` are type errors, even on non-literal values. */
export type StyleFor<S> = { readonly [K in keyof Style]?: K extends keyof S ? S[K] : never } & {
    readonly [VARIANTS]?: never;
};

/** A `style` prop: a style, a falsy value (skipped), or a nested array where later entries win per property. */
export type StyleValue<S = Style> = StyleFor<S> | false | null | undefined | readonly StyleValue<S>[];

/**
 * A named group of styles from `variants`, which components take to choose among. It is not a style itself. `K` names
 * the variants; `S` is the style each must be, such as `BoxStyle`.
 */
export type Variants<K extends string = string, S = Style> = { readonly [VARIANTS]: K } & {
    readonly [P in K]: StyleFor<S>;
};

type Exact<S> = { [P in keyof S]: P extends keyof Style ? S[P] : never };
type Checked<T> = { [K in keyof T]: T[K] extends { readonly [VARIANTS]: string } ? T[K] : Exact<T[K]> };

const ITEM_KEYS = [
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
] as const satisfies readonly (keyof ItemStyle)[];

const TEXT_KEYS = [
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
] as const satisfies readonly [keyof TextProps, string][];

/** Container properties and their `FlexStyle` fields. */
const FLEX_KEYS = [
    ["display", "display"],
    ["direction", "direction"],
    ["wrap", "wrap"],
    ["justify", "justify"],
    ["align", "align"],
    ["alignContent", "align_content"],
    ["gap", "gap"],
    ["padding", "padding"],
    ["width", "width"],
    ["height", "height"],
    ["minWidth", "min_width"],
    ["minHeight", "min_height"],
    ["maxWidth", "max_width"],
    ["maxHeight", "max_height"],
    ["aspectRatio", "aspect_ratio"],
    ["columns", "columns"],
    ["rows", "rows"],
    ["flow", "auto_flow"],
] as const satisfies readonly [keyof ContainerStyle, keyof FlexStyle][];

const CONTAINER_KEYS: readonly string[] = ["frame", "text", ...FLEX_KEYS.map(([key]) => key)];

/** The style properties each primitive accepts at runtime. */
export const STYLE_KEYS: Readonly<Record<string, readonly string[]>> = {
    box: [...ITEM_KEYS, ...CONTAINER_KEYS],
    text: [...ITEM_KEYS, ...TEXT_KEYS.map(([key]) => key)],
    item: ITEM_KEYS,
    switch: [...ITEM_KEYS, "text"],
    collection: [...ITEM_KEYS, "frame"],
    section: ["frame", "text"],
    case: CONTAINER_KEYS,
    hud: CONTAINER_KEYS.filter((key) => key !== "width" && key !== "height"),
};

const ALL_KEYS: ReadonlySet<string> = new Set(Object.values(STYLE_KEYS).flat());

function checkStyle(style: unknown, label: string): void {
    if (typeof style !== "object" || style === null || Array.isArray(style) || VARIANTS in style) {
        throw new Error(`${label} must be a style object`);
    }
    for (const key of Object.keys(style)) {
        if (!ALL_KEYS.has(key)) {
            throw new Error(`${label} has unknown style property \`${key}\``);
        }
    }
}

/**
 * Named styles, typed exactly as written. Each entry is a style or a `variants` group; unknown properties are errors.
 */
export function create<T extends Record<string, Style | Variants>>(styles: T & Checked<T>): T {
    const out: Fields = {};
    for (const [name, entry] of Object.entries(styles as Fields)) {
        if (typeof entry !== "object" || entry === null || !(VARIANTS in entry)) {
            checkStyle(entry, `style \`${name}\``);
        }
        out[name] = Object.isFrozen(entry) ? entry : Object.freeze({ ...(entry as Fields) });
    }
    return Object.freeze(out) as T;
}

/**
 * A named group of styles, such as `{ base, selected }`, that a component chooses among. The names mean nothing to
 * Window; components declare the ones they take with `Variants<"base" | "selected">`.
 */
export function variants<T extends Record<string, Style>>(
    styles: T & { [K in keyof T]: Exact<T[K]> },
): { readonly [VARIANTS]: keyof T & string } & Readonly<T> {
    const out: Fields = {};
    for (const [name, style] of Object.entries(styles as Fields)) {
        checkStyle(style, `variant \`${name}\``);
        out[name] = Object.freeze({ ...(style as Fields) });
    }
    return Object.freeze({ ...out, [VARIANTS]: Object.keys(out).join("|") }) as never;
}

/** Merges a `style` value: falsy entries are skipped, arrays flatten, and later entries win per property. */
export function mergeStyle(value: unknown, allowed: readonly string[], label: string): Fields {
    const out: Fields = {};
    const add = (entry: unknown): void => {
        if (entry === false || entry === null || entry === undefined || entry === 0 || entry === "") {
            return;
        }
        if (Array.isArray(entry)) {
            for (const item of entry as unknown[]) {
                add(item);
            }
            return;
        }
        if (typeof entry !== "object") {
            throw new Error(`${label} style entries must be styles, arrays, or falsy values`);
        }
        if (VARIANTS in entry) {
            throw new Error(`${label} style is a variants group; pass one of its variants, such as \`style.base\``);
        }
        for (const [key, field] of Object.entries(entry)) {
            if (!allowed.includes(key)) {
                throw new Error(`${label} does not accept style property \`${key}\``);
            }
            if (field !== undefined) {
                out[key] = field;
            }
        }
    };
    add(value);
    return out;
}

/** JSX props with `style` merged under them: explicit props win, and undefined ones fall back to the style. */
export function withStyle<P extends { style?: unknown }>(
    props: P,
    allowed: readonly string[],
    component: string,
): Omit<P, "style"> {
    const { style, ...rest } = props;
    if (style === undefined) {
        return rest;
    }
    const out = mergeStyle(style, allowed, `<${component}>`);
    for (const [key, value] of Object.entries(rest)) {
        if (value !== undefined) {
            out[key] = value;
        }
    }
    return out as Omit<P, "style">;
}

/** The element fields of text props. */
export function textFields(props: Fields): Fields {
    const out: Fields = {};
    for (const [from, to] of TEXT_KEYS) {
        if (props[from] !== undefined) {
            out[to] = props[from];
        }
    }
    return out;
}

/** The `FlexStyle` of container props, which may also be written as `FlexStyle` fields. */
export function flexStyle(props: Fields): FlexStyle | undefined {
    const out: Fields = {};
    for (const [from, to] of FLEX_KEYS) {
        const value = props[from] ?? props[to];
        if (value !== undefined) {
            out[to] = value;
        }
    }
    return Object.keys(out).length === 0 ? undefined : (out as FlexStyle);
}

/** `FlexStyle` field names, which raw boxes accept in `style` beside container props. */
export const FLEX_STYLE_KEYS: readonly string[] = FLEX_KEYS.map(([, key]) => key);

export function spanOf(span: ItemStyle["span"]): [number | undefined, number | undefined] {
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

/** The `layout` field of item props, or none. */
export function layout(props: ItemStyle): { layout?: ItemLayout } {
    const [columns, rows] = spanOf(props.span);
    const fields: Fields = {
        grow: typeof props.grow === "boolean" ? Number(props.grow) : props.grow,
        shrink: props.shrink,
        basis: props.basis,
        align_self: props.alignSelf,
        justify_self: props.justifySelf,
        margin: props.margin,
        position: props.absolute === true ? "absolute" : undefined,
        top: props.top,
        right: props.right,
        bottom: props.bottom,
        left: props.left,
        column: props.col ?? line(props.at?.[0], columns),
        row: props.row ?? line(props.at?.[1], rows),
        translate: props.translate,
    };
    const item = Object.fromEntries(Object.entries(fields).filter(([, value]) => value !== undefined));
    return Object.keys(item).length === 0 ? {} : { layout: item as ItemLayout };
}

/** Fills unset text fields of the text in `node` without modifying reusable children. */
export function cascade<T>(node: T, style: TextProps | undefined): T {
    if (style === undefined) {
        return node;
    }
    const defaults = textFields(style as Fields);
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

/**
 * A raw element with its `style` field applied: explicit fields win, `layout` merges per property, and `text` fills
 * the text inside. `kind` selects the primitive's style properties; boxes and cases take `FlexStyle` fields too.
 */
export function applyStyle(out: Fields, kind: string, label: string): Fields {
    const { style, ...rest } = out;
    if (style === undefined) {
        return rest;
    }
    const box = kind === "box" || kind === "case";
    const allowed = box ? [...STYLE_KEYS[kind]!, ...FLEX_STYLE_KEYS] : STYLE_KEYS[kind]!;
    const merged = mergeStyle(style, allowed, label);
    const result: Fields = { ...rest };
    if (merged.frame !== undefined && result.frame === undefined) {
        result.frame = merged.frame;
    }
    if (box) {
        const flex = flexStyle(merged);
        if (flex !== undefined) {
            result.style = flex;
        }
    }
    const item = layout(merged).layout;
    if (item !== undefined) {
        result.layout = { ...item, ...(rest.layout as Fields | undefined) };
    }
    if (kind === "text") {
        const fontSet = result.font !== undefined || result.small_caps !== undefined;
        for (const [key, value] of Object.entries(textFields(merged))) {
            if (result[key] === undefined && !((key === "font" || key === "small_caps") && fontSet)) {
                result[key] = value;
            }
        }
    }
    return cascade(result, merged.text as TextProps | undefined);
}
