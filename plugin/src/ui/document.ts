/** The compiled document's element and definition types, as the compiler reads them. */
import type { ClickAction, HandleKind } from "../bind/handles.ts";
import type {
    BoxStyle,
    CaseStyle,
    CollectionStyle,
    ItemStyle,
    SectionStyle,
    StyleValue,
    SwitchStyle,
    TextStyle,
} from "./styles.ts";
import type { Color, Var, VarTheme } from "./tokens.ts";
import type {
    Art,
    ArtRef,
    AutoLength,
    ContainerKind,
    Edges,
    FixedLength,
    FlexAlign,
    FlexJustify,
    GridFlow,
    GridLine,
    HudChannel,
    HudShader,
    Insets,
    Length,
    SlotList,
    SlotPatternInput,
    SlotRectClaim,
    SlotRectPatternOptions,
    SlotSection,
    TextAlign,
    TextFont,
    TextOverflow,
    Tooltip,
} from "./types.ts";

/** The authored name of an element, shown in errors and in the inspector. */
export interface DebugName {
    debug_name?: string;
}

/** Text styling shared by labels and dynamic text slots. */
export interface TextStyleOptions {
    align?: TextAlign;
    /** Hex color, e.g. "#ffffff". */
    color?: string | Var<Color>;
    shadow?: boolean;
    bold?: boolean;
    italic?: boolean;
    underlined?: boolean;
    strikethrough?: boolean;
    obfuscated?: boolean;
    /** A font name from `defineWindows({ fonts })`; text is measured and drawn with its glyphs. */
    font?: string;
    /** Shorthand for `font: "small_caps"`: Window's bundled small capitals, digits, and symbols. */
    small_caps?: boolean;
}

/** Options of elements that can be children of a flex or section element. */
export interface FlexItemOptions {
    /** Item layout inside a `flex`, `grid`, `section`, or switch case parent; rejected elsewhere. */
    layout?: ItemLayout;
}

export interface SpriteOptions extends FlexItemOptions, DebugName {
    x?: number;
    y?: number;
    style?: StyleValue<ItemStyle>;
}

export interface SpriteSlotOptions extends FlexItemOptions, DebugName {
    x?: number;
    y?: number;
    width: number;
    height: number;
    align?: TextAlign;
    style?: StyleValue<ItemStyle>;
}

export interface ItemOptions extends FlexItemOptions, DebugName {
    /** Backing inventory slots populated by this item. Integers mean container slots. */
    slots?: SlotList;
    /** Slot-space pattern for backing slots. */
    pattern?: SlotPatternInput;
    /** Inline slot-space rect transform. */
    transform?: SlotRectPatternOptions;
    style?: StyleValue<ItemStyle>;
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
    style?: StyleValue<CollectionStyle>;
}

export interface AnvilInputOptions extends DebugName {
    /** Initial contents of the vanilla anvil rename field. */
    initial?: string;
    /** Optional item model for the input-slot seed item. */
    item_model?: string;
}

export interface TextOptions extends TextStyleOptions, FlexItemOptions, DebugName {
    /** Fixed width; dynamic text without one must be laid out by a box, which it grows to fill. */
    width?: number;
    x?: number;
    y?: number;
    style?: StyleValue<TextStyle>;
}
export type LabelOptions = TextOptions;

export interface SlotOptions extends TextOptions {
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

/** The JSON of a handle an element reads. */
export type HandleJson = { readonly kind: HandleKind; readonly id: string } & Readonly<Record<string, unknown>>;

export interface SpriteElement extends SpriteOptions {
    type: "sprite";
    /** Inline art drawn at its own size. */
    art: Art;
}
export interface SpriteSlotElement extends SpriteSlotOptions {
    type: "sprite_slot";
    handle: HandleJson;
}
export interface ItemElement extends ItemOptions {
    type: "item";
    handle: HandleJson;
}
export interface CollectionElement extends CollectionOptions {
    type: "collection";
    handle: HandleJson;
}
export interface AnvilInputElement extends AnvilInputOptions {
    type: "anvil_input";
    handle: HandleJson;
}
export interface LabelElement extends LabelOptions {
    type: "label";
    text: string;
}
export interface SlotElement extends SlotOptions {
    type: "slot";
    handle: HandleJson;
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
    style?: StyleValue<ItemStyle>;
}
export interface RegionElement extends Omit<RegionOptions, "on_click"> {
    type: "region";
    on_click?: HandleJson;
}

/** Container style of a flex element. */
export interface FlexStyle {
    display?: "flex" | "grid";
    direction?: "row" | "column" | "row-reverse" | "column-reverse";
    wrap?: boolean;
    justify?: FlexJustify;
    align?: FlexAlign;
    align_content?: FlexJustify;
    gap?: FixedLength | [FixedLength, FixedLength] | Var<number>;
    padding?: Edges<FixedLength> | Var<number>;
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
    /** Overrides vars for this box and everything inside it. */
    theme?: VarTheme;
    children?: Element[];
}

/** Options of `box`: `style` takes styles from `create` (later entries win per property) instead of `FlexStyle`. */
export interface BoxOptions extends Omit<FlexOptions, "style"> {
    style?: StyleValue<BoxStyle>;
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
    style?: StyleValue<SectionStyle>;
    children?: Element[];
}

export interface SectionElement extends SectionOptions {
    type: "section";
    section: SlotSection;
}

/** A case of `switchOn`, or the shown case of `show`: a flex box that stretches to the switch unless sized. */
export interface CaseOptions extends DebugName {
    frame?: ArtRef;
    /** Styles from `create`; `direction` defaults to `"column"`. */
    style?: StyleValue<CaseStyle>;
    /** Overrides vars for this case and everything inside it. */
    theme?: VarTheme;
    children?: Element[];
}

export interface SwitchOptions extends FlexItemOptions, DebugName {
    x?: number;
    y?: number;
    style?: StyleValue<SwitchStyle>;
}

/** One case of a switch: a flex box that fills the switch. */
export interface CaseElement extends DebugName {
    type: "case";
    /** Value the switch binding returns to draw this case. */
    value: string;
    frame?: ArtRef;
    style?: FlexStyle;
    theme?: VarTheme;
    children?: Element[];
}

/** Cases stacked in one box sized to the largest case. The runtime draws only the case its handle selects. */
export interface SwitchElement extends SwitchOptions {
    type: "switch";
    handle: HandleJson;
    children: CaseElement[];
}

export type Element = (
    | SpriteElement
    | SpriteSlotElement
    | ItemElement
    | CollectionElement
    | AnvilInputElement
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
    style?: StyleValue<SectionStyle>;
    /** Overrides vars for the whole window. */
    theme?: VarTheme;
    children?: Element[];
    debug_name?: string;
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
    style?: StyleValue<SectionStyle>;
    /** Overrides vars for the whole HUD. */
    theme?: VarTheme;
    children?: Element[];
    debug_name?: string;
}

export interface WindowDocument {
    /** Text fonts by name, which text selects with `font`. */
    fonts?: Record<string, TextFont>;
    /** Runtime sprites by name, generated into Kotlin's `WindowSprite`. */
    sprites?: Record<string, Art>;
    windows?: Window[];
    huds?: Hud[];
    window?: Window;
    hud?: Hud;
}
