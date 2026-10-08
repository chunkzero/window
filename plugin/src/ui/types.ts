import type { Color, Var } from "./tokens.ts";

export type TextAlign = "left" | "center" | "right";
export type ContainerKind =
    | "generic_9x1"
    | "generic_9x2"
    | "generic_9x3"
    | "generic_9x4"
    | "generic_9x5"
    | "generic_9x6"
    | "anvil";
export type HudChannel = "actionbar" | "bossbar" | "sidebar";
export type SlotArea = "container" | "player";
export type SlotSection = "container" | "player" | "hotbar";
export type SlotRectClaim = "none" | "all" | "unowned";
export type GeneratedKind =
    | "panel"
    | "button"
    | "slot"
    | "hazard_bar"
    | "vent"
    | "badge"
    | "chip"
    | "slot_cell"
    | "hazard"
    | "corner_cap";

/** Per-edge pixel values; a single number applies to all edges. */
export interface InsetsEdges {
    top?: number;
    right?: number;
    bottom?: number;
    left?: number;
}
export type Insets = number | InsetsEdges;

export interface SlotRef {
    /** Backing inventory area. */
    area: SlotArea;
    /** Slot index in the selected area. */
    index: number;
}

export interface SlotRange {
    /** Backing inventory area. */
    area: SlotArea;
    /** First slot index, inclusive. */
    first: number;
    /** Last slot index, inclusive. */
    last: number;
}

export type SlotList = (SlotRef | SlotRange | number)[];

export interface SlotRectPattern {
    kind: "rect";
    section: SlotSection;
    /** Zero-based grid x in the selected section. */
    x: number;
    /** Zero-based grid y in the selected section. */
    y: number;
    /** Width in slots. */
    width: number;
    /** Height in slots. */
    height: number;
}

export interface SlotSlotsPattern {
    kind: "slots";
    section: SlotSection;
    /** Local slot indices in the selected section. */
    slots: number[];
}

export interface SlotGridPattern {
    kind: "grid";
    section: SlotSection;
    /** Zero-based grid x of the first repeated cell group. */
    x: number;
    /** Zero-based grid y of the first repeated cell group. */
    y: number;
    /** Number of cell groups per row. */
    columns: number;
    /** Number of cell-group rows. */
    rows: number;
    /** Width of each cell group in slots. */
    cell_width: number;
    /** Height of each cell group in slots. */
    cell_height: number;
}

export type SlotPattern = SlotRectPattern | SlotSlotsPattern | SlotGridPattern;

/** Raw pattern input; an omitted `section` is "container" and an omitted `kind` is "rect". */
export type SlotPatternInput =
    | (Omit<SlotRectPattern, "kind" | "section"> & { kind?: "rect"; section?: SlotSection })
    | (Omit<SlotSlotsPattern, "section"> & { section?: SlotSection })
    | (Omit<SlotGridPattern, "section"> & { section?: SlotSection });

export interface SlotRectPatternOptions {
    /** Defaults to "container". */
    section?: SlotSection;
    /** Zero-based grid x. */
    x: number;
    /** Zero-based grid y. */
    y: number;
    /** Width in slots. */
    width: number;
    /** Height in slots. */
    height: number;
}

export interface SlotSlotsPatternOptions {
    /** Defaults to "container". */
    section?: SlotSection;
}

export interface SlotGridPatternOptions {
    /** Defaults to "container". */
    section?: SlotSection;
    /** Zero-based grid x of the first repeated cell group. */
    x: number;
    /** Zero-based grid y of the first repeated cell group. */
    y: number;
    /** Number of cell groups per row. */
    columns: number;
    /** Number of cell-group rows. */
    rows: number;
    /** Width of each cell group in slots. */
    cell_width: number;
    /** Height of each cell group in slots. */
    cell_height: number;
}

export interface Tooltip {
    title: string;
    lines?: string[];
}

export interface GeneratedStyle {
    /** Generated kind when `texture` is omitted. */
    kind?: GeneratedKind;
    /** Hex fill color, e.g. "#087fae". */
    fill?: string;
    /** Hex border color. */
    border_color?: string;
    /** Border width in pixels. */
    border_width?: number;
    /** Corner radius in pixels. */
    radius?: number;
    /** Inner bevel/depth in pixels. */
    inset_depth?: number;
    /** Top/left bevel color. */
    highlight_color?: string;
    /** Bottom/right bevel color. */
    shadow_color?: string;
    /** Decorative accent color. */
    accent_color?: string;
    /** Panel, button, and slot frames only: a 2px bar along the inner bottom edge, e.g. a selected-tab lamp. */
    indicator_color?: string;
    /** Hazard stripe color. */
    stripe_color?: string;
    /** Hazard stripe shadow color. */
    stripe_shadow_color?: string;
    /** Hazard stripe width in pixels. */
    stripe_width?: number;
}

/** The optional name of inline art, which prefixes its content-hashed resource name. */
export interface ArtName {
    /** Must match ^[a-z0-9][a-z0-9_/-]*$; `industrial/button` names the art `art/industrial-button-<hash>`. */
    name?: string;
}

/** Inline bitmap art: `texture(path, options)`. */
export interface TextureArt extends ArtName {
    readonly art: "texture";
    /** Pack-source-relative texture path, or a texture id such as "example:item/coin.png". */
    texture: string;
    /** Nine-slice inset pixels, used where the art stretches over a box. */
    insets?: Insets;
    /** Size drawn as an image; set with `height`, or omit both for the texture's own size. */
    width?: number;
    height?: number;
}

/** `T` with each color field also taking a `Var<Color>` and each number field a `Var<number>`. */
export type Tokenized<T> = {
    [K in keyof T]: K extends "kind" ? T[K] : T[K] | (NonNullable<T[K]> extends number ? Var<number> : Var<Color>);
};

/** Generated art's style and, for images, its size. Fields may be vars, resolved under the theme where it is drawn. */
export type ShapeStyle = Tokenized<
    GeneratedStyle & {
        /** Size drawn as an image; a frame stretches over its box instead. */
        width?: number;
        height?: number;
    }
>;

/** Inline generated art: `shape(style, options)`. */
export type ShapeArt = ShapeStyle & ArtName & { readonly art: "shape" };

/**
 * Art authored in place: a box frame stretched over its laid-out size, or an image drawn at its own size. Identical
 * art is shared.
 */
export type Art = TextureArt | ShapeArt;

/** Inline art, or art `derive`d from tokens. */
export type ArtRef = Art | Var<Art>;

/**
 * A bitmap text font: a glyph sheet of 8px-tall cells that each `chars` row maps, left to right. Cells sit on the text
 * line like vanilla `ascii.png` cells: seven rows above the baseline and one below.
 */
export interface TextFont {
    /** Pack-source-relative glyph sheet path. */
    texture: string;
    /** One string per sheet row; use "\u0000" for empty cells. Unmapped characters fall back to vanilla glyphs. */
    chars: string[];
}

export interface HudShaderPoint {
    x?: number;
    y?: number;
}

export interface HudShader {
    /** Source text baseline surface top from bottom of the vanilla HUD channel. */
    source_bottom?: number;
    /** Normalized GUI target origin; 0.0 is left/top, 1.0 is right/bottom. */
    origin?: HudShaderPoint;
    /** Normalized point inside the HUD placed on the target origin. */
    anchor?: HudShaderPoint;
    /** Flat alias for origin.x. */
    origin_x?: number;
    /** Flat alias for origin.y. */
    origin_y?: number;
    /** Flat alias for anchor.x. */
    anchor_x?: number;
    /** Flat alias for anchor.y. */
    anchor_y?: number;
    /** Horizontal nudge from the normalized origin, in GUI pixels. */
    x?: number;
    /** Vertical nudge from the normalized origin, in GUI pixels. */
    y?: number;
    /** @deprecated use x/y */
    offset_x?: number;
    /** @deprecated use x/y */
    offset_y?: number;
}

/** How a bound text slot shortens content wider than its width: `"ellipsis"` truncates it and appends "…". */
export type TextOverflow = "ellipsis";

/** A CSS length: pixels, a percentage, or a sizing keyword. */
export type FixedLength = number | `${number}%`;
export type AutoLength = FixedLength | "auto";
export type Length = AutoLength | "min-content" | "max-content" | "fit-content" | "stretch";
/** Per-edge lengths; a single value applies to all edges. */
export type Edges<T extends Length = AutoLength> = T | { top?: T; right?: T; bottom?: T; left?: T };
/** A one-based grid line, or `{ start?, end?, span? }`. Negative lines count from the end. */
export type GridLine = number | { start?: number; end?: number; span?: number };
export type FlexAlign = "start" | "end" | "center" | "stretch" | "baseline";
export type FlexJustify = "start" | "end" | "center" | "stretch" | "between" | "around" | "evenly";
export type GridFlow = "row" | "column" | "row-dense" | "column-dense";
