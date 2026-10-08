import {
    action,
    collection,
    derive,
    flag,
    industrial,
    raw,
    selection,
    sprite,
    text,
    texture,
    toggle,
} from "#plugins/window";
import type {
    Child,
    Element,
    LabelOptions,
    SelectionHandle,
    SlotOptions,
    TextHandle,
    ToggleHandle,
} from "#plugins/window";

import { coin } from "./sprites.ts";

const style = { color: "#ffffff", shadow: true, small_caps: true };

const say = (value: string, opts: LabelOptions = {}) => raw.text(value, { ...style, ...opts });
const bound = (handle: TextHandle, opts: SlotOptions = {}) => raw.text(handle, { ...style, ...opts });

const ventSlot = derive((get) => ({
    ...get(industrial.art.recess),
    border_width: 0,
    inset_depth: 0,
    width: 6,
    height: 2,
}));

const category = selection("category", ["all", "gear", "magic"]);
const sort = selection("sort", ["featured", "price", "name"]);
const products = collection("products", { selectable: true });
const hasPrice = flag("has_price");

/** A button that is disabled while `can_<id>` is false. */
function actionButton(id: string, tooltip: string, children: Child, opts: { span?: number; accent?: boolean } = {}) {
    return industrial.Button({
        onClick: action(id),
        enabled: flag(`can_${id}`),
        span: opts.span ?? 1,
        tooltip,
        disabled: { tooltip: tooltip + " unavailable" },
        ...(opts.accent === true ? { frame: industrial.art.buttonAccent } : {}),
        children,
    });
}

/** A row of tabs, each showing its `<name>_<value>_label` text. */
function tabs<V extends string>(bind: SelectionHandle<V>, entries: [V, string][]) {
    return industrial.Tabs({
        bind,
        span: 3,
        children: entries.map(([value, tooltip]) =>
            industrial.Tab({ value, tooltip, children: bound(text(`${bind.id}_${value}_label`)) }),
        ),
    });
}

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function lampToggle(bind: ToggleHandle, title: string, tooltip: string): Element {
    return industrial.Toggle({
        bind,
        span: 3,
        on: { tooltip: tooltip + " on" },
        off: { tooltip: tooltip + " off" },
        children: [
            raw.image(sprite(bind.id + "_lamp", { only: ["lamp_on", "lamp_off"] }), { width: 4, height: 4 }),
            say(title),
        ],
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
    frame: industrial.art.shell,
    bleed: { top: 1, right: 4, bottom: 7, left: 4 },
    children: [
        industrial.Header({
            padding: { top: 3 },
            align: "start",
            children: raw.box({
                frame: industrial.art.recess,
                style: { justify: "center", align: "center", width: 116, height: 12 },
                children: [say("Foundry Exchange")],
            }),
        }),

        raw.box({ frame: industrial.art.panel, x: 4, y: 136, style: { width: 168, height: 82 } }),
        raw.box({ frame: industrial.art.hazardBar, x: -4, y: 223, style: { width: 184, height: 6 } }),
        industrial.Row({ x: 52, y: 131, gap: 4, children: Array.from({ length: 8 }, () => raw.image(ventSlot)) }),
        ...rivets.map(([x, y]) => raw.image(industrial.art.rivet, { x, y })),

        raw.section("container", {
            frame: industrial.art.panel,
            outset: { top: 2, right: 3, bottom: 3, left: 3 },
            children: [
                ...tabs(category, [
                    ["all", "All items"],
                    ["gear", "Gear"],
                    ["magic", "Magic"],
                ]),

                industrial.Collection({ bind: products, rows: 3 }),

                actionButton("previous", "Previous page", bound(text("previous_label")), { span: 3 }),
                industrial.Row({
                    span: 3,
                    frame: industrial.art.recess,
                    children: bound(text("page"), { align: "center" }),
                }),
                actionButton("next", "Next page", bound(text("next_label")), { span: 3 }),

                industrial.Row({
                    span: 9,
                    frame: industrial.art.recess,
                    padding: { left: 5, right: 4 },
                    gap: 4,
                    children: [
                        bound(text("selection"), { color: "#ffb20b" }),
                        bound(text("price"), { width: 40, align: "right" }),
                        industrial.Show({ when: hasPrice, children: raw.image(coin) }),
                    ],
                }),
            ],
        }),

        raw.section("player", {
            children: [
                ...tabs(sort, [
                    ["featured", "Top picks first"],
                    ["price", "Price sort"],
                    ["name", "Name sort"],
                ]),

                lampToggle(toggle("favorites"), "Favs", "Favorites only"),
                lampToggle(toggle("affordable"), "Afford", "Affordable only"),
                industrial.Button({
                    onClick: action("search"),
                    span: 2,
                    tooltip: "Search the catalog",
                    children: say("Find"),
                }),
                actionButton("clear_search", "Clear search", raw.image(texture("window/icons/clear.png"))),

                industrial.Row({
                    span: 9,
                    frame: industrial.art.recess,
                    padding: { left: 5, right: 4 },
                    gap: 4,
                    children: [
                        bound(text("status"), { color: "#bceeff" }),
                        bound(text("balance"), { width: 48, align: "right", color: "#ffb20b" }),
                        raw.image(coin),
                    ],
                }),
            ],
        }),

        raw.section("hotbar", {
            children: [
                actionButton(
                    "buy",
                    "Buy selected item",
                    bound(text("buy_label"), { color: "#2a1200", shadow: false }),
                    {
                        span: 6,
                        accent: true,
                    },
                ),
                industrial.Button({ close: true, span: 3, tooltip: "Close market", children: say("Exit") }),
            ],
        }),
    ],
});
