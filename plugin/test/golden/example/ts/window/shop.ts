import { button, choice, collection, label, panel, pattern, slot, slotRects, toggle, ui } from "#plugins/window";

const hitbox = "window:gui/hitbox";
function choiceStates(label: string) {
    return {
        selected: { item_model: hitbox, tooltip: label + " selected" },
        unselected: { item_model: hitbox, tooltip: "Select " + label },
    };
}
function toggleStates(label: string) {
    return { on: { item_model: hitbox, tooltip: label + " on" }, off: { item_model: hitbox, tooltip: label + " off" } };
}
function enabledStates(label: string) {
    return {
        enabled: { item_model: hitbox, tooltip: label },
        disabled: { item_model: hitbox, tooltip: label + " unavailable" },
    };
}
export default ui({
    name: "shop",
    container: "generic_9x6",
    bleed: { top: 10, right: 40, bottom: 12, left: 40 },
    children: [
        panel({ frame: "shell", x: -38, y: -8, width: 252, height: 31 }),
        panel({ frame: "header", x: -34, y: -4, width: 244, height: 25 }),
        panel({ frame: "hazard_bar", x: -34, y: -6, width: 14, height: 27 }),
        panel({ frame: "hazard_bar", x: 196, y: -6, width: 14, height: 27 }),
        panel({ frame: "vent", x: -15, y: -1, width: 31, height: 16 }),
        panel({ frame: "vent", x: 160, y: -1, width: 31, height: 16 }),
        panel({ frame: "shell", x: -28, y: 21, width: 232, height: 208 }),
        panel({ frame: "rail", x: -24, y: 25, width: 10, height: 194 }),
        panel({ frame: "rail", x: 190, y: 25, width: 10, height: 194 }),
        panel({ frame: "surface", x: -14, y: 14, width: 204, height: 119 }),
        panel({ frame: "panel", x: -14, y: 132, width: 204, height: 88 }),
        panel({ frame: "recess", x: 58, y: 92, width: 60, height: 19 }),
        panel({ frame: "recess", x: 3, y: 111, width: 170, height: 17 }),
        panel({ frame: "recess", x: 3, y: 178, width: 170, height: 17 }),
        panel({ frame: "hazard_bar", x: -24, y: 219, width: 224, height: 10 }),
        label("FOUNDRY EXCHANGE", { x: 24, y: 5, width: 128, align: "center", color: "#ffffff", bold: true }),
        slot("balance", { x: 120, y: 181, width: 48, align: "right", color: "#ffb20b", bold: true }),
        choice("category_all", {
            frame: "tab",
            transform: { section: "container", x: 0, y: 0, width: 3, height: 1 },
            states: choiceStates("All items"),
            children: [slot("category_all_label", { color: "#ffffff", bold: true })],
        }),
        choice("category_gear", {
            frame: "tab",
            transform: { section: "container", x: 3, y: 0, width: 3, height: 1 },
            states: choiceStates("Gear"),
            children: [slot("category_gear_label", { color: "#ffffff", bold: true })],
        }),
        choice("category_magic", {
            frame: "tab",
            transform: { section: "container", x: 6, y: 0, width: 3, height: 1 },
            states: choiceStates("Magic"),
            children: [slot("category_magic_label", { color: "#ffffff", bold: true })],
        }),
        collection("products", {
            frame: "slot_cell",
            pattern: pattern.rect({ section: "container", x: 1, y: 1, width: 7, height: 3 }),
        }),
        button("previous", {
            frame: "button",
            transform: { section: "container", x: 1, y: 4, width: 2, height: 1 },
            states: enabledStates("Previous page"),
            children: [slot("previous_label", { color: "#ffffff", bold: true })],
        }),
        slot("page", { x: 61, y: 95, width: 54, align: "center", color: "#ffffff", bold: true }),
        button("next", {
            frame: "button",
            transform: { section: "container", x: 6, y: 4, width: 2, height: 1 },
            states: enabledStates("Next page"),
            children: [slot("next_label", { color: "#ffffff", bold: true })],
        }),
        slot("selection", { x: 8, y: 113, width: 160, align: "center", color: "#ffb20b", bold: true }),
        choice("sort_featured", {
            frame: "tab",
            transform: { section: "player", x: 0, y: 0, width: 3, height: 1 },
            states: choiceStates("Featured sort"),
            children: [slot("sort_featured_label", { color: "#ffffff" })],
        }),
        choice("sort_price", {
            frame: "tab",
            transform: { section: "player", x: 3, y: 0, width: 3, height: 1 },
            states: choiceStates("Price sort"),
            children: [slot("sort_price_label", { color: "#ffffff" })],
        }),
        choice("sort_name", {
            frame: "tab",
            transform: { section: "player", x: 6, y: 0, width: 3, height: 1 },
            states: choiceStates("Name sort"),
            children: [slot("sort_name_label", { color: "#ffffff" })],
        }),
        toggle("favorites", {
            frame: "button",
            transform: { section: "player", x: 0, y: 1, width: 4, height: 1 },
            states: toggleStates("Favorites only"),
            children: [slot("favorites_label", { color: "#ffffff" })],
        }),
        toggle("affordable", {
            frame: "button",
            transform: { section: "player", x: 5, y: 1, width: 4, height: 1 },
            states: toggleStates("Affordable only"),
            children: [slot("affordable_label", { color: "#ffffff" })],
        }),
        slot("status", { x: 8, y: 181, width: 108, align: "left", color: "#bceeff" }),
        button("search", {
            frame: "button",
            transform: { section: "hotbar", x: 0, y: 0, width: 3, height: 1 },
            tooltip: "Search the catalog",
            children: [label("SEARCH", { color: "#ffffff", bold: true })],
        }),
        button("buy", {
            frame: "button_accent",
            transform: { section: "hotbar", x: 3, y: 0, width: 4, height: 1 },
            states: enabledStates("Buy selected item"),
            children: [slot("buy_label", { color: "#1b1607", bold: true })],
        }),
        button("exit", {
            frame: "button_danger",
            transform: { section: "hotbar", x: 7, y: 0, width: 2, height: 1 },
            default: "close",
            tooltip: "Close market",
            children: [label("EXIT", { color: "#ffffff", bold: true })],
        }),
        slotRects("container_fill", {
            transform: { section: "container", x: 0, y: 0, width: 9, height: 6 },
            claim: "unowned",
        }),
        slotRects("player_fill", {
            transform: { section: "player", x: 0, y: 0, width: 9, height: 3 },
            claim: "unowned",
        }),
        slotRects("hotbar_fill", {
            transform: { section: "hotbar", x: 0, y: 0, width: 9, height: 1 },
            claim: "unowned",
        }),
    ],
});
