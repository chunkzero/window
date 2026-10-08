import type { ClickAction } from "./handles.ts";

export type CrossAlign = "start" | "center" | "end";
export type TextAlign = "left" | "center" | "right";
export type ButtonDefault = "close";
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

export interface ButtonState {
    /** Item model id, e.g. "example:gui/shop_button_active". */
    item_model?: string;
    /** Frame drawn over the button instead of its `frame` in this state. */
    frame?: ArtRef;
    /** Sprite drawn at the button's top-left corner in this state. */
    sprite?: ArtRef;
    tooltip?: string | Tooltip;
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

type NoGeneratedStyle = { [K in keyof GeneratedStyle]?: never };

export interface BitmapFrame extends NoGeneratedStyle {
    /** Pack-source-relative texture path for bitmap nine-slice frames. */
    texture: string;
    /** Nine-slice inset pixels. */
    insets?: Insets;
}

export interface GeneratedFrame extends GeneratedStyle {
    texture?: never;
    insets?: never;
}

export type Frame = BitmapFrame | GeneratedFrame;

/** Bitmap sprite; `width` and `height` are set together or both omitted. */
export type BitmapSprite = NoGeneratedStyle & {
    /** Pack-source-relative texture path, or a font texture id such as "example:tool/wooden_pickaxe.png". */
    texture: string;
} & ({ width: number; height: number } | { width?: never; height?: never });

export interface GeneratedSprite extends GeneratedStyle {
    texture?: never;
    width: number;
    height: number;
}

export type SpriteDef = BitmapSprite | GeneratedSprite;

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

/** Inline generated art: `shape(style, options)`. */
export interface ShapeArt extends GeneratedStyle, ArtName {
    readonly art: "shape";
    /** Size drawn as an image; a frame stretches over its box instead. */
    width?: number;
    height?: number;
}

/**
 * Art authored in place: a box frame stretched over its laid-out size, or an image drawn at its own size. Identical
 * art is shared, and it never clashes with theme names.
 */
export type Art = TextureArt | ShapeArt;

/** A theme frame or sprite name, or inline art. */
export type ArtRef = string | Art;

/** The authored name of an element, shown in errors and in the inspector. */
export interface DebugName {
    debug_name?: string;
}

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

export interface Theme {
    frames?: Record<string, Frame>;
    sprites?: Record<string, SpriteDef>;
    /** Text fonts elements select with `font`. A `small_caps` entry replaces the bundled small-caps font. */
    fonts?: Record<string, TextFont>;
    /** Palette colors by name as "#rrggbb", generated into Kotlin as `WindowColors` `TextColor` constants. */
    colors?: Record<string, string>;
}

/** Base colors for `presets.industrial()`; bevels and borders are derived from these fills. */
export interface IndustrialPresetOptions {
    border_color?: string;
    highlight_color?: string;
    shell_fill?: string;
    panel_fill?: string;
    recess_fill?: string;
    slot_fill?: string;
    button_fill?: string;
    selected_fill?: string;
    disabled_fill?: string;
    accent_color?: string;
    danger_color?: string;
    confirm_color?: string;
    hud_fill?: string;
    stripe_fill?: string;
    stripe_color?: string;
    lamp_color?: string;
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

/** Text styling shared by labels and dynamic text slots. */
export interface TextStyleOptions {
    align?: TextAlign;
    /** Hex color, e.g. "#ffffff". */
    color?: string;
    shadow?: boolean;
    bold?: boolean;
    italic?: boolean;
    underlined?: boolean;
    strikethrough?: boolean;
    obfuscated?: boolean;
    /** A theme font name; text is measured and drawn with its glyphs. */
    font?: string;
    /** Shorthand for `font: "small_caps"`: Window's bundled small capitals, digits, and symbols. */
    small_caps?: boolean;
}

/** Options of elements that can be children of a flex or section element. */
export interface FlexItemOptions {
    /** Item layout inside a `flex`, `grid`, `section`, or switch case parent; rejected elsewhere. */
    layout?: ItemLayout;
}

export interface PanelOptions extends FlexItemOptions {
    frame: ArtRef;
    width: number;
    height: number;
    x?: number;
    y?: number;
    padding?: number;
    children?: Element[];
}

export interface LayoutOptions extends FlexItemOptions {
    x?: number;
    y?: number;
    gap?: number;
    padding?: number;
    align?: CrossAlign;
    children?: Element[];
}
export type RowOptions = LayoutOptions;
export type ColumnOptions = LayoutOptions;

export interface SpriteOptions extends FlexItemOptions, DebugName {
    x?: number;
    y?: number;
}

/**
 * Index of one entry in an indexed binding family: a number, or `[row, column]`. Entries sharing a binding name are
 * flattened to `<name>_<i>` (or `<name>_<row>_<column>`) and generated as one Kotlin member taking the index. A family
 * must cover every index from zero up to its largest one.
 */
export type BindingIndex = number | readonly [number, number];

export interface SpriteSlotOptions extends FlexItemOptions, DebugName {
    /** Places this sprite slot in an indexed binding family. */
    index?: BindingIndex;
    x?: number;
    y?: number;
    width: number;
    height: number;
    align?: TextAlign;
    /** Theme sprite name drawn by default. */
    sprite?: string;
}

export interface ButtonOptions extends FlexItemOptions {
    frame?: ArtRef;
    /** Required unless `pattern` or `transform` is set, or the element is placed by a section. */
    width?: number;
    /** Required unless `pattern` or `transform` is set, or the element is placed by a section. */
    height?: number;
    x?: number;
    y?: number;
    /** Explicit backing inventory slots. Integers mean container slots. */
    slots?: SlotList;
    /** Slot-space pattern for rect/slots. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    default?: ButtonDefault;
    tooltip?: string | Tooltip;
    states?: Record<string, ButtonState>;
    padding?: number;
    /** Unpositioned children are centered in the padded button content rect. */
    children?: Element[];
}

export interface ToggleOptions extends ButtonOptions {
    /** Requires `on` and `off` states. */
    states: Record<string, ButtonState> & { on: ButtonState; off: ButtonState };
}

export interface ChoiceOptions extends ButtonOptions {
    /** Requires `selected` and `unselected` states. */
    states: Record<string, ButtonState> & { selected: ButtonState; unselected: ButtonState };
}

export interface HotspotOptions extends FlexItemOptions {
    /** Required unless `pattern` or `transform` is set, or the element is placed by a section. */
    width?: number;
    /** Required unless `pattern` or `transform` is set, or the element is placed by a section. */
    height?: number;
    x?: number;
    y?: number;
    /** Explicit backing inventory slots. Integers mean container slots. */
    slots?: SlotList;
    /** Slot-space pattern for rect/slots. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    /** Required unless `states` is set. */
    tooltip?: string | Tooltip;
    /** Required unless `tooltip` is set. */
    states?: Record<string, ButtonState>;
}

export interface ItemOptions extends FlexItemOptions, DebugName {
    /** Backing inventory slots populated by this item. Integers mean container slots. */
    slots?: SlotList;
    /** Slot-space pattern for backing slots. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    /** One-based index into the enclosing repeater cell's own slots. Repeater children only; mutually exclusive with slots/pattern/transform. */
    cell_slot?: number;
}

export interface CollectionOptions extends FlexItemOptions, DebugName {
    /** Optional frame drawn once around every collection cell. */
    frame?: ArtRef;
    /** Optional sprite drawn over the 18x18 box of the cell the runtime marks selected. */
    selected_sprite?: ArtRef;
    /** Ordered backing inventory slots for this collection. Integers mean container slots. */
    slots?: SlotList;
    /** Slot-space pattern for backing slots. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    /** Whether clicks route to a generated handler. Defaults to true. */
    action?: boolean;
}

export interface AnvilInputOptions extends DebugName {
    /** Initial contents of the vanilla anvil rename field. */
    initial?: string;
    /** Optional item model for the input-slot seed item. */
    item_model?: string;
}

export interface SlotRectsOptions extends FlexItemOptions {
    /** Optional frame to draw once per slot. Omit for claim-only slot rects. */
    frame?: string;
    /** Slot-space pattern to draw and optionally claim. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    /** Defaults to "none"; "unowned" claims only slots not owned by controls. */
    claim?: SlotRectClaim;
}

export interface RepeaterOptions extends FlexItemOptions {
    /** Optional frame drawn once per repeated cell group. */
    frame?: string;
    /** Slot-space pattern whose cells are repeated. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    padding?: number;
    /** Template children placed inside each repeated cell. */
    children?: Element[];
}

export interface TextOptions extends TextStyleOptions, FlexItemOptions, DebugName {
    /** Required for dynamic slots except direct, unpositioned button children, which fill the button content width. */
    width?: number;
    x?: number;
    y?: number;
}
export type LabelOptions = TextOptions;
/** How a bound text slot shortens content wider than its width: `"ellipsis"` truncates it and appends "…". */
export type TextOverflow = "ellipsis";

export interface SlotOptions extends TextOptions {
    /** Places this slot in an indexed binding family. */
    index?: BindingIndex;
    /** Shortens content wider than the slot at runtime. */
    overflow?: TextOverflow;
    /**
     * Wraps content at spaces onto at most this many lines, ellipsizing the last; at least 1. The slot is
     * `(lines - 1) * line_height + 8` pixels tall, and the used lines are vertically centered in it.
     */
    lines?: number;
    /** Distance between the tops of consecutive lines in pixels; defaults to 9, the vanilla line spacing. */
    line_height?: number;
}

export interface PanelElement extends PanelOptions {
    type: "panel";
}
export interface RowElement extends RowOptions {
    type: "row";
}
export interface ColumnElement extends ColumnOptions {
    type: "column";
}
export interface SpriteElement extends SpriteOptions {
    type: "sprite";
    /** A theme sprite name; set either `name` or `art`. */
    name?: string;
    /** Inline art drawn at its own size. */
    art?: Art;
}
export interface SpriteSlotElement extends SpriteSlotOptions {
    type: "sprite_slot";
    name: string;
}
export interface ButtonElement extends ButtonOptions {
    type: "button";
    name: string;
    /** The widget kind that produced this button, such as `"toggle"`, for diagnostics. */
    source?: string;
}
export interface HotspotElement extends HotspotOptions {
    type: "hotspot";
    name: string;
}
export interface ItemElement extends ItemOptions {
    type: "item";
    name: string;
}
export interface CollectionElement extends CollectionOptions {
    type: "collection";
    name: string;
}
export interface AnvilInputElement extends AnvilInputOptions {
    type: "anvil_input";
    name: string;
}
export interface SlotRectsElement extends SlotRectsOptions {
    type: "slot_rects";
    name: string;
}
export interface RepeaterElement extends RepeaterOptions {
    type: "repeater";
    name: string;
}
export interface LabelElement extends LabelOptions {
    type: "label";
    text: string;
}
export interface SlotElement extends SlotOptions {
    type: "slot";
    name: string;
}

/**
 * An inventory region: the slots its rect covers route clicks to `on_click` and show its hitbox item. Without a size
 * it fills its parent box, or in a section its grid area. Switch cases may hold regions over the same slots.
 */
export interface RegionOptions extends FlexItemOptions, DebugName {
    /** What a click does; omit for a hover-only region. */
    on_click?: ClickAction;
    tooltip?: string | Tooltip;
    /** Item model of the hitbox item filling its slots. */
    item_model?: string;
    /** Fixed size, set together; omit both to fill the parent box. */
    width?: number;
    height?: number;
}
export interface RegionElement extends RegionOptions {
    type: "region";
}

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

/** Container style of a flex element. */
export interface FlexStyle {
    display?: "flex" | "grid";
    direction?: "row" | "column" | "row-reverse" | "column-reverse";
    wrap?: boolean;
    justify?: FlexJustify;
    align?: FlexAlign;
    align_content?: FlexJustify;
    gap?: FixedLength | [FixedLength, FixedLength];
    padding?: Edges<FixedLength>;
    width?: Length;
    height?: Length;
    min_width?: AutoLength;
    min_height?: AutoLength;
    max_width?: AutoLength;
    max_height?: AutoLength;
    aspect_ratio?: number;
    /** Grid columns: a count of equal `1fr` tracks, or track sizes such as `24`, `"1fr"`, or `"auto"`. */
    columns?: number | (AutoLength | "min-content" | "max-content" | `${number}fr`)[];
    rows?: number | (AutoLength | "min-content" | "max-content" | `${number}fr`)[];
    auto_flow?: GridFlow;
}

/** Item layout of a child of a flex or section element. */
export interface ItemLayout {
    grow?: number;
    shrink?: number;
    basis?: Length;
    align_self?: FlexAlign;
    justify_self?: FlexAlign;
    margin?: Edges;
    position?: "relative" | "absolute";
    top?: AutoLength;
    right?: AutoLength;
    bottom?: AutoLength;
    left?: AutoLength;
    min_width?: AutoLength;
    min_height?: AutoLength;
    max_width?: AutoLength;
    max_height?: AutoLength;
    column?: GridLine;
    row?: GridLine;
    /** Pixel offset applied after layout to the element and its subtree. */
    translate?: [number, number];
}

export interface FlexOptions extends FlexItemOptions, DebugName {
    /** Explicit pixel position; inside another flex box this positions the box absolutely. */
    x?: number;
    y?: number;
    /** Frame stretched over the box's laid-out size. */
    frame?: ArtRef;
    style?: FlexStyle;
    children?: Element[];
}

export interface FlexElement extends FlexOptions {
    type: "flex";
}

export interface SectionOptions extends DebugName {
    frame?: ArtRef;
    /** How far the frame extends past the slot boxes. `section()` defaults it to 3 when a frame is set. */
    outset?: Insets;
    /** Claim for section slots no child owns. Defaults to "unowned". */
    claim?: SlotRectClaim;
    flow?: GridFlow;
    children?: Element[];
}

export interface SectionElement extends SectionOptions {
    type: "section";
    section: SlotSection;
}

/** A case of `switchOn`, or the shown case of `show`: a flex box that stretches to the switch unless sized. */
export interface CaseOptions extends DebugName {
    frame?: ArtRef;
    /** Box layout; `direction` defaults to `"column"`. */
    style?: FlexStyle;
    children?: Element[];
}

export interface SwitchOptions extends FlexItemOptions, DebugName {
    /** Places this switch in an indexed binding family; every entry must have the same case values. */
    index?: BindingIndex;
    x?: number;
    y?: number;
}

export type ShowOptions = CaseOptions & SwitchOptions;

/** One case of a switch: a flex box that fills the switch. */
export interface CaseElement extends DebugName {
    type: "case";
    /** Value the switch binding returns to draw this case. */
    value: string;
    frame?: ArtRef;
    style?: FlexStyle;
    children?: Element[];
}

/**
 * Visual cases stacked in one box sized to the largest case. The runtime draws only the case its binding names.
 * Cases hold static art and text/icon bindings, not slot-bound controls.
 */
export interface SwitchElement extends SwitchOptions {
    type: "switch";
    /** Binding name. Kotlin gets a Boolean for exactly `true`/`false` cases, otherwise an enum of the case values. */
    name: string;
    children: CaseElement[];
}

export type Element = (
    | PanelElement
    | RowElement
    | ColumnElement
    | SpriteElement
    | SpriteSlotElement
    | ButtonElement
    | HotspotElement
    | ItemElement
    | CollectionElement
    | AnvilInputElement
    | SlotRectsElement
    | RepeaterElement
    | LabelElement
    | SlotElement
    | RegionElement
    | FlexElement
    | SectionElement
    | SwitchElement
) & { layout?: ItemLayout };

export interface Window {
    name: string;
    container: ContainerKind;
    /** Visual overflow allowed outside the container GUI. */
    bleed?: Insets;
    /** Frame drawn first, over the GUI rect grown by `bleed`. */
    frame?: ArtRef;
    children?: Element[];
}

export interface Hud {
    name: string;
    channel?: HudChannel;
    /** Canvas width; omit with `height` to size the HUD to its children. */
    width?: number;
    height?: number;
    /** Visual overflow allowed outside the HUD canvas. */
    bleed?: Insets;
    /** Frame drawn first, over the HUD rect grown by `bleed`. */
    frame?: ArtRef;
    shader?: HudShader;
    children?: Element[];
}

export interface WindowDocument {
    theme?: Theme;
    /** Runtime sprites by name, generated into Kotlin's `WindowSprite`. */
    sprites?: Record<string, Art>;
    windows?: Window[];
    huds?: Hud[];
    window?: Window;
    hud?: Hud;
}
