/** The faces industrial controls draw: their layered styles, label text, and placement props. */
import type { Child, DebugProps } from "../../ui/components.ts";
import { assign } from "../../ui/styles.ts";
import type { ContainerStyle, ItemStyle, StyleFor, StyleValue, TextProps, Variants } from "../../ui/styles.ts";
import type { ArtRef, FixedLength, Tooltip } from "../../ui/types.ts";

export type Layer = StyleValue<ContainerStyle>;

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
export interface FaceProps {
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
export interface ControlProps extends ItemStyle, DebugProps, FaceProps {}

/** The props of a face that draw it rather than place its control. */
export const FACE_KEYS: readonly string[] = [
    "frame",
    "tooltip",
    "itemModel",
    "padding",
    "gap",
    "text",
    "children",
    "style",
];

const FACE: StyleFor<ContainerStyle> = { direction: "row", justify: "center", align: "center", gap: 2 };

/** The objects in `value`, a nested array of them. */
export function flat(value: unknown, out: Record<string, unknown>[] = []): Record<string, unknown>[] {
    if (Array.isArray(value)) {
        value.forEach((entry) => flat(entry, out));
    } else if (typeof value === "object" && value !== null) {
        out.push(value as Record<string, unknown>);
    }
    return out;
}

/**
 * The style and text of a face: `base` layers, then the explicit props, then the `state` layers and the state's
 * frame. Labels are centered unless a layer or `text` aligns them.
 */
export function face(
    base: readonly Layer[],
    state: readonly Layer[],
    props: FaceProps,
    current: StateProps = {},
): { style: Layer[]; text: TextProps } {
    const explicit = { frame: props.frame, padding: props.padding, gap: props.gap };
    const style: Layer[] = [FACE, ...base, explicit, ...state, { frame: current.frame }];
    const text: Record<string, unknown> = { align: "center" };
    for (const fields of [...flat(style).map((entry) => entry["text"]), props.text]) {
        if (typeof fields === "object" && fields !== null) {
            assign(text, fields as Record<string, unknown>);
        }
    }
    return { style, text: text as TextProps };
}

/** The defined props of `props` that place the control rather than draw it. */
export function placement(props: object, drawn: readonly string[]): ItemStyle & DebugProps {
    const entries = Object.entries(props).filter(([key, value]) => value !== undefined && !drawn.includes(key));
    return Object.fromEntries(entries) as ItemStyle & DebugProps;
}
