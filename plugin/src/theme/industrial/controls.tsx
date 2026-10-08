/**
 * Industrial's controls, built from the public primitives. Each takes a `style` of the variants it names and defaults
 * to industrial's styles, so `style` is only for overrides.
 */
import { builtin } from "../../bind/handles.ts";
import { Box, Case, Region, Switch } from "../../ui/components.ts";
import type { ClickAction, Condition, Selection, Toggle } from "../../bind/handles.ts";
import type { Element } from "../../ui/document.ts";
import type { Child, DebugProps } from "../../ui/components.ts";
import type { BoxStyle, CaseStyle, ItemStyle, StyleValue } from "../../ui/styles.ts";
import type { ArtRef, Tooltip } from "../../ui/types.ts";
import { FACE_KEYS, face, flat, placement } from "./face.ts";
import type { ControlProps, FaceProps, Layer, StateProps, VariantStyle } from "./face.ts";
import { styles } from "./styles.ts";

/** A control's box: in a section, its slot cells inset by 1px, so its face covers the slot interiors. */
function control(props: object, drawn: readonly string[], content: Child): Element {
    return (
        <Box {...placement(props, [...FACE_KEYS, ...drawn])} padding={1}>
            {content}
        </Box>
    ) as Element;
}

/** The region of a face: it takes the face's clicks and shows its tooltip. */
function region(props: FaceProps, current: StateProps, onClick: ClickAction | undefined): Child {
    return (
        <Region
            onClick={onClick}
            tooltip={current.tooltip ?? props.tooltip}
            itemModel={current.itemModel ?? props.itemModel}
        />
    );
}

/** A face without states, filling its control. */
function plain(base: readonly Layer[], props: FaceProps, onClick: ClickAction): Child {
    const { style, text } = face(base, [], props);
    return (
        <Box grow={1} style={style as StyleValue<BoxStyle>} text={text}>
            {props.children}
            {region(props, {}, onClick)}
        </Box>
    );
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
    return (
        <Case value={value} style={style as StyleValue<CaseStyle>} text={text}>
            {props.children}
            {region(props, current, onClick)}
        </Case>
    );
}

/** A control showing its `true` face while `when` holds and its `false` face otherwise. */
function switched(props: object, drawn: readonly string[], when: Condition, faces: [Child, Child]): Element {
    return control(
        props,
        drawn,
        <Switch bind={when} grow={1}>
            {faces}
        </Switch>,
    );
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
        throw new Error("<Button> requires exactly one of `onClick` or `close`");
    }
    if (props.disabled !== undefined && props.enabled === undefined) {
        throw new Error("<Button disabled> requires `enabled`");
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
                throw new Error(`<Tabs> children must be <Tab> elements with values of \`${bind.id}\``);
            }
            tabs.set(tab.value, tab);
        }
    }
    return bind.values.map((value, i) => {
        const tab = tabs.get(value);
        const content = typeof children === "function" ? children(value, i) : tab?.children;
        return (
            <Choice
                {...rest}
                tooltip={tab?.tooltip ?? tooltip?.(value) ?? labelText(content)}
                itemModel={typeof itemModel === "function" ? itemModel(value) : itemModel}
                bind={bind}
                value={value}
                selected={{ frame: selectedSprite }}
                unselected={{ frame: sprite }}
            >
                {content}
            </Choice>
        ) as Element;
    });
}

export interface HotspotProps extends ItemStyle, DebugProps {
    tooltip: string | Tooltip;
    itemModel?: string;
}

/** A tooltip over its slots; it takes no clicks. */
export function Hotspot(props: HotspotProps): Element {
    return (<Region {...props} />) as Element;
}
