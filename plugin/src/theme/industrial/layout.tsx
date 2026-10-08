/** Industrial's layout components: boxes with preset alignment, the inventory sections, and conditional content. */
import { Box, Case, Section, Switch } from "../../ui/components.ts";
import type { Condition } from "../../bind/handles.ts";
import type { Element } from "../../ui/document.ts";
import type { BoxProps, Child, DebugProps, SectionProps } from "../../ui/components.ts";
import type { CaseStyle, ItemStyle } from "../../ui/styles.ts";
import type { ArtRef } from "../../ui/types.ts";

/** The opened container's slot grid. */
export function Container(props: SectionProps): Element {
    return (<Section {...props} of="container" />) as Element;
}

/** The player's 9x3 main inventory grid. */
export function Player(props: SectionProps): Element {
    return (<Section {...props} of="player" />) as Element;
}

/** The player's 9x1 hotbar grid. */
export function Hotbar(props: SectionProps): Element {
    return (<Section {...props} of="hotbar" />) as Element;
}

/** A horizontal box whose children are vertically centered by default. */
export function Row(props: BoxProps): Element {
    return (<Box {...props} style={[{ align: "center" }, props.style]} direction="row" />) as Element;
}

export function Column(props: BoxProps): Element {
    return (<Box {...props} direction="column" />) as Element;
}

export function Grid(props: BoxProps): Element {
    return (<Box {...props} display="grid" />) as Element;
}

/** A box that centers its children on both axes. */
export function Center(props: BoxProps): Element {
    return (<Box {...props} style={[{ justify: "center", align: "center" }, props.style]} />) as Element;
}

/** Flexible empty space; in a section it skips `span` slots. */
export function Spacer(props: ItemStyle & DebugProps): Element {
    return (<Box {...props} grow={props.grow ?? 1} />) as Element;
}

/** The title strip above the container grid, centering its children. */
export function Header(props: BoxProps): Element {
    return (
        <Box
            {...props}
            style={[{ justify: "center", align: "center", gap: 4 }, props.style]}
            x={0}
            y={0}
            width={176}
            height={17}
        />
    ) as Element;
}

export interface ShowProps extends ItemStyle, DebugProps, Omit<CaseStyle, "width" | "height"> {
    when: Condition;
    frame?: ArtRef;
    children?: Child;
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

/** Draws its children only while `when` holds; their space is always reserved. */
export function Show(props: ShowProps): Element {
    const { when, children, frame, debugName, ...rest } = props;
    const item: Record<string, unknown> = {};
    const box: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(rest)) {
        if (value !== undefined) {
            (ITEM_KEYS.has(key) ? item : box)[key] = value;
        }
    }
    return (
        <Switch {...(item as ItemStyle)} debugName={debugName} bind={when}>
            <Case {...box} frame={frame} value="true">
                {children}
            </Case>
            <Case value="false" />
        </Switch>
    ) as Element;
}
