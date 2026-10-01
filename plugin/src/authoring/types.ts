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
    /** Theme sprite name drawn in this state. */
    sprite?: string;
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

export interface Theme {
    frames?: Record<string, Frame>;
    sprites?: Record<string, SpriteDef>;
}

export interface IndustrialPresetOptions {
    shell_fill?: string;
    panel_fill?: string;
    surface_fill?: string;
    deep_fill?: string;
    border_color?: string;
    highlight_color?: string;
    cyan_dark?: string;
    accent_color?: string;
    accent_light?: string;
    accent_dark?: string;
    danger_color?: string;
    danger_dark?: string;
    confirm_color?: string;
    confirm_dark?: string;
    header_fill?: string;
    title_fill?: string;
    button_fill?: string;
    tab_fill?: string;
    slot_fill?: string;
    search_fill?: string;
    vent_fill?: string;
    vent_color?: string;
    badge_accent?: string;
    stripe_color?: string;
    stripe_shadow_color?: string;
    stripe_width?: number;
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
}

export interface PanelOptions {
    frame: string;
    width: number;
    height: number;
    x?: number;
    y?: number;
    padding?: number;
    children?: Element[];
}

export interface LayoutOptions {
    x?: number;
    y?: number;
    gap?: number;
    padding?: number;
    align?: CrossAlign;
    children?: Element[];
}
export type RowOptions = LayoutOptions;
export type ColumnOptions = LayoutOptions;

export interface SpriteOptions {
    x?: number;
    y?: number;
}

export interface SpriteSlotOptions {
    x?: number;
    y?: number;
    width: number;
    height: number;
    align?: TextAlign;
    /** Theme sprite name drawn by default. */
    sprite?: string;
}

export interface ButtonOptions {
    frame?: string;
    /** Required unless `pattern` or `transform` is set. */
    width?: number;
    /** Required unless `pattern` or `transform` is set. */
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

export interface HotspotOptions {
    /** Required unless `pattern` or `transform` is set. */
    width?: number;
    /** Required unless `pattern` or `transform` is set. */
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

export interface ItemOptions {
    /** Backing inventory slots populated by this item. Integers mean container slots. */
    slots?: SlotList;
    /** Slot-space pattern for backing slots. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    /** One-based index into the enclosing repeater cell's own slots. Repeater children only; mutually exclusive with slots/pattern/transform. */
    cell_slot?: number;
}

export interface CollectionOptions {
    /** Optional frame drawn once around every collection cell. */
    frame?: string;
    /** Ordered backing inventory slots for this collection. Integers mean container slots. */
    slots?: SlotList;
    /** Slot-space pattern for backing slots. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    /** Whether clicks route to a generated handler. Defaults to true. */
    action?: boolean;
}

export interface AnvilInputOptions {
    /** Initial contents of the vanilla anvil rename field. */
    initial?: string;
    /** Optional item model for the input-slot seed item. */
    item_model?: string;
}

export interface SlotRectsOptions {
    /** Optional frame to draw once per slot. Omit for claim-only slot rects. */
    frame?: string;
    /** Slot-space pattern to draw and optionally claim. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    /** Defaults to "none"; "unowned" claims only slots not owned by controls. */
    claim?: SlotRectClaim;
}

export interface RepeaterOptions {
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

export interface TextOptions extends TextStyleOptions {
    /** Required for dynamic slots except direct, unpositioned button children, which fill the button content width. */
    width?: number;
    x?: number;
    y?: number;
}
export type LabelOptions = TextOptions;
export type SlotOptions = TextOptions;

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
    name: string;
}
export interface SpriteSlotElement extends SpriteSlotOptions {
    type: "sprite_slot";
    name: string;
}
export interface ButtonElement extends ButtonOptions {
    type: "button";
    name: string;
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

export type Element =
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
    | SlotElement;

export interface Window {
    name: string;
    container: ContainerKind;
    /** Visual overflow allowed outside the container GUI. */
    bleed?: Insets;
    children?: Element[];
}

export interface Hud {
    name: string;
    channel?: HudChannel;
    width: number;
    height: number;
    /** Visual overflow allowed outside the HUD canvas. */
    bleed?: Insets;
    shader?: HudShader;
    children?: Element[];
}

export interface WindowDocument {
    theme?: Theme;
    windows?: Window[];
    huds?: Hud[];
    window?: Window;
    hud?: Hud;
}
