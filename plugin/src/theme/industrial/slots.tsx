/** Industrial's slot controls: collections, slot frames, and repeated cells. */
import { Box, Collection as CollectionPrimitive, Items, Region } from "../../ui/components.ts";
import type { Action, Indexed, Items as ItemsHandle } from "../../bind/handles.ts";
import type { Element } from "../../ui/document.ts";
import type { Child, CollectionProps as CollectionPrimitiveProps, DebugProps } from "../../ui/components.ts";
import type { BoxStyle, ItemStyle, StyleValue, TextProps } from "../../ui/styles.ts";
import type { ArtRef, Tooltip } from "../../ui/types.ts";
import { placement } from "./face.ts";
import type { Layer, VariantStyle } from "./face.ts";
import { styles } from "./styles.ts";

export interface CollectionProps extends Omit<CollectionPrimitiveProps, "name" | "bind" | "frame" | "selected"> {
    bind: NonNullable<CollectionPrimitiveProps["bind"]>;
    /** Frame of every cell; defaults to industrial's slot. */
    frame?: ArtRef;
    /** Art over the selected cell; defaults to industrial's selected slot. */
    selected?: ArtRef;
}

/** A scrolling item collection drawn as industrial slots; by default it spans the full section width. */
export function Collection(props: CollectionProps): Element {
    return (
        <CollectionPrimitive
            {...props}
            selected={props.selected ?? styles.collection.selected}
            style={[{ frame: styles.collection.frame }, props.style]}
        />
    ) as Element;
}

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
    const cells = Array.from({ length: columns * rows }, () => <Box frame={frame} />);
    return (
        <Box {...item} span={span} display="grid" columns={columns} rows={rows}>
            {cells}
            {claim === "all" ? <Region /> : null}
        </Box>
    ) as Element;
}

export interface RepeaterProps extends ItemStyle, DebugProps {
    /** Size of one cell in slots. */
    cell: [number, number];
    columns: number;
    rows: number;
    /** An action indexed by cell; a click calls it with the clicked cell's index. */
    onClick?: Indexed<Action, readonly [number]>;
    tooltip?: string | Tooltip | ((i: number) => string | Tooltip);
    /** Items indexed by cell, shown in each cell's `itemSlot`; clicks on it still call `onClick`. */
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
        throw new Error(`<Repeater itemSlot> must be between 1 and ${width * height}`);
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
        { frame, padding },
    ];
    const cells = Array.from({ length: columns * rows }, (_, i) => {
        let stack: Child = null;
        if (item !== undefined) {
            const [x, y] = [(itemSlot - 1) % width, Math.floor((itemSlot - 1) / width)];
            stack = (
                <Box absolute left={x * SLOT} top={y * SLOT} width={SLOT - 2} height={SLOT - 2}>
                    <Items bind={item.at(i)} />
                </Box>
            );
        }
        return (
            <Box padding={1}>
                <Box grow={1} style={layers as StyleValue<BoxStyle>} text={text}>
                    {typeof children === "function" ? children(i) : children}
                    {stack}
                    <Region onClick={onClick?.at(i)} tooltip={typeof tooltip === "function" ? tooltip(i) : tooltip} />
                </Box>
            </Box>
        );
    });
    return (
        <Box span={[columns * width, rows * height]} {...placed} display="grid" columns={columns} rows={rows}>
            {cells}
        </Box>
    ) as Element;
}
