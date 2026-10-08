import { raw } from "#plugins/window";
import type { ButtonState, Element, LabelOptions, SlotOptions } from "#plugins/window";

const text = { color: "#ffffff", shadow: true, small_caps: true };
const span = (columns: number) => ({ column: { span: columns } });

const say = (value: string, opts: LabelOptions = {}) => raw.label(value, { ...text, ...opts });
const bound = (name: string, opts: SlotOptions = {}) => raw.slot(name, { ...text, ...opts });

/** Button content: a centered row; text in it is centered. */
function content(...children: Element[]): Element[] {
    return [raw.flex({ style: { direction: "row", justify: "center", align: "center", gap: 2 }, children })];
}

function enabled(tooltip: string, sprite: string): Record<string, ButtonState> {
    return {
        enabled: { tooltip, sprite },
        disabled: { tooltip: tooltip + " unavailable", sprite: sprite + "_disabled" },
    };
}

/** A row of tab choices named `<name>_<value>`, each showing its `<name>_<value>_label` text slot. */
function tabs(name: string, entries: [value: string, tooltip: string][]): Element[] {
    return entries.map(([value, tooltip]) =>
        raw.choice(`${name}_${value}`, {
            states: {
                selected: { tooltip, sprite: "tab_selected" },
                unselected: { tooltip, sprite: "tab" },
            },
            layout: span(3),
            children: content(bound(`${name}_${value}_label`, { align: "center" })),
        }),
    );
}

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function lampToggle(name: string, title: string, tooltip: string): Element {
    return raw.toggle(name, {
        frame: "button",
        states: { on: { tooltip: tooltip + " on" }, off: { tooltip: tooltip + " off" } },
        layout: span(3),
        children: content(raw.spriteSlot(name + "_lamp", { width: 4, height: 4 }), say(title, { align: "center" })),
    });
}

const rivets: [number, number][] = [
    [-2, 6],
    [173, 6],
    [-2, 129],
    [173, 129],
];

export default raw.ui({
    name: "shop",
    container: "generic_9x6",
    frame: "shell",
    bleed: { top: 1, right: 4, bottom: 7, left: 4 },
    children: [
        raw.flex({
            x: 0,
            y: 0,
            style: { justify: "center", align: "start", gap: 4, padding: { top: 3 }, width: 176, height: 17 },
            children: [
                raw.flex({
                    frame: "recess",
                    style: { justify: "center", align: "center", width: 116, height: 12 },
                    children: [say("Foundry Exchange")],
                }),
            ],
        }),

        raw.flex({ frame: "panel", x: 4, y: 136, style: { width: 168, height: 82 } }),
        raw.flex({ frame: "hazard_bar", x: -4, y: 223, style: { width: 184, height: 6 } }),
        raw.flex({
            x: 52,
            y: 131,
            style: { direction: "row", align: "center", gap: 4 },
            children: Array.from({ length: 8 }, () => raw.sprite("vent_slot")),
        }),
        ...rivets.map(([x, y]) => raw.sprite("rivet", { x, y })),

        raw.section("container", {
            frame: "panel",
            outset: { top: 2, right: 3, bottom: 3, left: 3 },
            children: [
                ...tabs("category", [
                    ["all", "All items"],
                    ["gear", "Gear"],
                    ["magic", "Magic"],
                ]),

                raw.collection("products", {
                    frame: "slot",
                    selected_sprite: "slot_selected",
                    layout: { column: { start: 1, end: -1 }, row: { span: 3 } },
                }),

                raw.button("previous", {
                    states: enabled("Previous page", "action"),
                    layout: span(3),
                    children: content(bound("previous_label", { align: "center" })),
                }),
                raw.flex({
                    frame: "recess",
                    style: { direction: "row", align: "center" },
                    layout: span(3),
                    children: [bound("page", { align: "center" })],
                }),
                raw.button("next", {
                    states: enabled("Next page", "action"),
                    layout: span(3),
                    children: content(bound("next_label", { align: "center" })),
                }),

                raw.flex({
                    frame: "recess",
                    style: { direction: "row", align: "center", gap: 4, padding: { left: 5, right: 4 } },
                    layout: span(9),
                    children: [
                        bound("selection", { color: "#ffb20b" }),
                        bound("price", { width: 40, align: "right" }),
                        raw.show("has_price", { children: [raw.sprite("coin")] }),
                    ],
                }),
            ],
        }),

        raw.section("player", {
            children: [
                ...tabs("sort", [
                    ["featured", "Top picks first"],
                    ["price", "Price sort"],
                    ["name", "Name sort"],
                ]),

                lampToggle("favorites", "Favs", "Favorites only"),
                lampToggle("affordable", "Afford", "Affordable only"),
                raw.button("search", {
                    frame: "button",
                    tooltip: "Search the catalog",
                    layout: span(2),
                    children: content(say("Find", { align: "center" })),
                }),
                raw.button("clear_search", {
                    states: enabled("Clear search", "clear_search"),
                    children: content(
                        raw.spriteSlot("clear_search_icon", { width: 8, height: 6, sprite: "icon_clear" }),
                    ),
                }),

                raw.flex({
                    frame: "recess",
                    style: { direction: "row", align: "center", gap: 4, padding: { left: 5, right: 4 } },
                    layout: span(9),
                    children: [
                        bound("status", { color: "#bceeff" }),
                        bound("balance", { width: 48, align: "right", color: "#ffb20b" }),
                        raw.sprite("coin"),
                    ],
                }),
            ],
        }),

        raw.section("hotbar", {
            children: [
                raw.button("buy", {
                    states: enabled("Buy selected item", "buy"),
                    layout: span(6),
                    children: content(bound("buy_label", { align: "center", color: "#2a1200", shadow: false })),
                }),
                raw.button("exit", {
                    frame: "button",
                    default: "close",
                    tooltip: "Close market",
                    layout: span(3),
                    children: content(say("Exit", { align: "center" })),
                }),
            ],
        }),
    ],
});
