import { action, collection, flag, industrial, pattern, raw, selection, sprite, text, toggle } from "#plugins/window";
import type { Child, SelectionHandle } from "#plugins/window";

import { coin, icons, ventSlot } from "./art.ts";

const hitbox = "window:gui/hitbox";
const style = { color: "#ffffff", shadow: true, small_caps: true };

const category = selection("category", ["all", "gear", "magic"]);
const sort = selection("sort", ["featured", "price", "name"]);

/** A choice of `bind` over three slots at `at` in its section, labeled by the text handle `<id>_<value>_label`. */
function choice<V extends string>(bind: SelectionHandle<V>, value: V, label: string, at: [number, number]) {
    return industrial.Choice({
        bind,
        value,
        span: 3,
        at,
        selected: { itemModel: hitbox, tooltip: label + " selected" },
        unselected: { itemModel: hitbox, tooltip: "Select " + label },
        children: raw.text(text(`${bind.id}_${value}_label`), style),
    });
}

/** A button running `id` that shows itself disabled, taking no clicks, while the flag `can_<id>` is false. */
function guarded(
    id: string,
    label: string,
    span: number,
    at: [number, number],
    children: Child,
): industrial.ButtonProps {
    return {
        onClick: action(id),
        enabled: flag(`can_${id}`),
        span,
        at,
        tooltip: label,
        itemModel: hitbox,
        disabled: { tooltip: label + " unavailable", itemModel: hitbox },
        children,
    };
}

/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function lampToggle(id: string, x: number, title: string, tooltip: string) {
    return industrial.Toggle({
        bind: toggle(id),
        span: 3,
        at: [x, 1],
        on: { itemModel: hitbox, tooltip: tooltip + " on" },
        off: { itemModel: hitbox, tooltip: tooltip + " off" },
        children: [
            raw.image(sprite(`${id}_lamp`), { x: 5, y: 6, width: 4, height: 4 }),
            raw.text(title, { ...style, x: 12, y: 4, width: 36, align: "center" }),
        ],
    });
}

export default raw.ui({
    name: "shop",
    container: "generic_9x6",
    bleed: { top: 1, right: 4, bottom: 7, left: 4 },
    children: [
        raw.box({ frame: industrial.art.shell, x: -4, y: -1, style: { width: 184, height: 230 } }),
        raw.box({ frame: industrial.art.panel, x: 4, y: 15, style: { width: 168, height: 113 } }),
        raw.box({ frame: industrial.art.panel, x: 4, y: 136, style: { width: 168, height: 82 } }),
        raw.box({ frame: industrial.art.recess, x: 30, y: 3, style: { width: 116, height: 12 } }),
        raw.box({ frame: industrial.art.recess, x: 62, y: 90, style: { width: 52, height: 16 } }),
        raw.box({ frame: industrial.art.recess, x: 8, y: 108, style: { width: 160, height: 16 } }),
        raw.box({ frame: industrial.art.recess, x: 8, y: 175, style: { width: 160, height: 16 } }),
        raw.box({ frame: industrial.art.hazardBar, x: -4, y: 223, style: { width: 184, height: 6 } }),
        ...[52, 62, 72, 82, 92, 102, 112, 122].map((x) => raw.image(ventSlot, { x, y: 131 })),
        raw.image(industrial.art.rivet, { x: -2, y: 6 }),
        raw.image(industrial.art.rivet, { x: 173, y: 6 }),
        raw.image(industrial.art.rivet, { x: -2, y: 129 }),
        raw.image(industrial.art.rivet, { x: 173, y: 129 }),
        raw.image(coin, { x: 156, y: 179 }),
        raw.text("Foundry Exchange", { ...style, x: 0, y: 6, width: 176, align: "center" }),
        raw.collection(collection("products", { selectable: true }), {
            frame: industrial.art.slot,
            selected_sprite: industrial.art.slotSelected,
            pattern: pattern.rect({ section: "container", x: 0, y: 1, width: 9, height: 3 }),
        }),
        raw.text(text("page"), { ...style, x: 62, y: 94, width: 52, align: "center" }),
        raw.text(text("selection"), { ...style, x: 12, y: 112, width: 100, color: "#ffb20b" }),
        raw.text(text("price"), { ...style, x: 112, y: 112, width: 40, align: "right" }),
        raw.image(sprite("price_coin"), { x: 156, y: 112, width: 8, height: 8 }),
        raw.text(text("status"), { ...style, x: 12, y: 179, width: 92, color: "#bceeff" }),
        raw.text(text("balance"), { ...style, x: 104, y: 179, width: 48, align: "right", color: "#ffb20b" }),
        raw.section("container", {
            children: [
                choice(category, "all", "All items", [0, 0]),
                choice(category, "gear", "Gear", [3, 0]),
                choice(category, "magic", "Magic", [6, 0]),
                industrial.Button(
                    guarded("previous", "Previous page", 3, [0, 4], raw.text(text("previous_label"), style)),
                ),
                industrial.Button(guarded("next", "Next page", 3, [6, 4], raw.text(text("next_label"), style))),
            ],
        }),
        raw.section("player", {
            children: [
                choice(sort, "featured", "Top picks first", [0, 0]),
                choice(sort, "price", "Price sort", [3, 0]),
                choice(sort, "name", "Name sort", [6, 0]),
                lampToggle("favorites", 0, "Favs", "Favorites only"),
                lampToggle("affordable", 3, "Afford", "Affordable only"),
                // A fixed width centers the label like the runtime does, rounding down.
                industrial.Button({
                    onClick: action("search"),
                    span: 2,
                    at: [6, 1],
                    tooltip: "Search the catalog",
                    children: raw.text("Find", { ...style, width: 34 }),
                }),
                industrial.Button(guarded("clear_search", "Clear search", 1, [8, 1], raw.image(icons.clear))),
            ],
        }),
        raw.section("hotbar", {
            children: [
                industrial.Button({
                    ...guarded(
                        "buy",
                        "Buy selected item",
                        6,
                        [0, 0],
                        raw.text(text("buy_label"), { ...style, color: "#2a1200", shadow: false }),
                    ),
                    style: { base: { frame: industrial.art.buttonAccent } },
                }),
                industrial.Button({
                    close: true,
                    span: 3,
                    at: [6, 0],
                    tooltip: "Close market",
                    children: raw.text("Exit", { ...style, width: 52 }),
                }),
            ],
        }),
    ],
});
