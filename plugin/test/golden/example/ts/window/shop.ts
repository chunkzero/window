import { pattern, raw } from "#plugins/window";

const hitbox = "window:gui/hitbox";
const text = { color: "#ffffff", shadow: true, small_caps: true };

function choiceStates(label: string) {
    return {
        selected: { item_model: hitbox, tooltip: label + " selected", sprite: "tab_selected" },
        unselected: { item_model: hitbox, tooltip: "Select " + label, sprite: "tab" },
    };
}
function enabledStates(label: string, sprite: string) {
    return {
        enabled: { item_model: hitbox, tooltip: label, sprite },
        disabled: { item_model: hitbox, tooltip: label + " unavailable", sprite: sprite + "_disabled" },
    };
}
/** A raised toggle with a lamp at its left and its label centered in the remaining width. */
function lampToggle(name: string, x: number, title: string, tooltip: string) {
    return raw.toggle(name, {
        frame: "button",
        transform: { section: "player", x, y: 1, width: 3, height: 1 },
        states: {
            on: { item_model: hitbox, tooltip: tooltip + " on" },
            off: { item_model: hitbox, tooltip: tooltip + " off" },
        },
        children: [
            raw.spriteSlot(name + "_lamp", { x: 5, y: 6, width: 4, height: 4 }),
            raw.label(title, { ...text, x: 12, y: 4, width: 36, align: "center" }),
        ],
    });
}
export default raw.ui({
    name: "shop",
    container: "generic_9x6",
    bleed: { top: 1, right: 4, bottom: 7, left: 4 },
    children: [
        raw.panel({ frame: "shell", x: -4, y: -1, width: 184, height: 230 }),
        raw.panel({ frame: "panel", x: 4, y: 15, width: 168, height: 113 }),
        raw.panel({ frame: "panel", x: 4, y: 136, width: 168, height: 82 }),
        raw.panel({ frame: "recess", x: 30, y: 3, width: 116, height: 12 }),
        raw.panel({ frame: "recess", x: 62, y: 90, width: 52, height: 16 }),
        raw.panel({ frame: "recess", x: 8, y: 108, width: 160, height: 16 }),
        raw.panel({ frame: "recess", x: 8, y: 175, width: 160, height: 16 }),
        raw.panel({ frame: "hazard_bar", x: -4, y: 223, width: 184, height: 6 }),
        ...[52, 62, 72, 82, 92, 102, 112, 122].map((x) => raw.sprite("vent_slot", { x, y: 131 })),
        raw.sprite("rivet", { x: -2, y: 6 }),
        raw.sprite("rivet", { x: 173, y: 6 }),
        raw.sprite("rivet", { x: -2, y: 129 }),
        raw.sprite("rivet", { x: 173, y: 129 }),
        raw.sprite("coin", { x: 156, y: 179 }),
        raw.label("Foundry Exchange", { ...text, x: 0, y: 6, width: 176, align: "center" }),
        raw.choice("category_all", {
            transform: { section: "container", x: 0, y: 0, width: 3, height: 1 },
            states: choiceStates("All items"),
            children: [raw.slot("category_all_label", text)],
        }),
        raw.choice("category_gear", {
            transform: { section: "container", x: 3, y: 0, width: 3, height: 1 },
            states: choiceStates("Gear"),
            children: [raw.slot("category_gear_label", text)],
        }),
        raw.choice("category_magic", {
            transform: { section: "container", x: 6, y: 0, width: 3, height: 1 },
            states: choiceStates("Magic"),
            children: [raw.slot("category_magic_label", text)],
        }),
        raw.collection("products", {
            frame: "slot",
            selected_sprite: "slot_selected",
            pattern: pattern.rect({ section: "container", x: 0, y: 1, width: 9, height: 3 }),
        }),
        raw.button("previous", {
            transform: { section: "container", x: 0, y: 4, width: 3, height: 1 },
            states: enabledStates("Previous page", "action"),
            children: [raw.slot("previous_label", text)],
        }),
        raw.slot("page", { ...text, x: 62, y: 94, width: 52, align: "center" }),
        raw.button("next", {
            transform: { section: "container", x: 6, y: 4, width: 3, height: 1 },
            states: enabledStates("Next page", "action"),
            children: [raw.slot("next_label", text)],
        }),
        raw.slot("selection", { ...text, x: 12, y: 112, width: 100, color: "#ffb20b" }),
        raw.slot("price", { ...text, x: 112, y: 112, width: 40, align: "right" }),
        raw.spriteSlot("price_coin", { x: 156, y: 112, width: 8, height: 8 }),
        raw.choice("sort_featured", {
            transform: { section: "player", x: 0, y: 0, width: 3, height: 1 },
            states: choiceStates("Top picks first"),
            children: [raw.slot("sort_featured_label", text)],
        }),
        raw.choice("sort_price", {
            transform: { section: "player", x: 3, y: 0, width: 3, height: 1 },
            states: choiceStates("Price sort"),
            children: [raw.slot("sort_price_label", text)],
        }),
        raw.choice("sort_name", {
            transform: { section: "player", x: 6, y: 0, width: 3, height: 1 },
            states: choiceStates("Name sort"),
            children: [raw.slot("sort_name_label", text)],
        }),
        lampToggle("favorites", 0, "Favs", "Favorites only"),
        lampToggle("affordable", 3, "Afford", "Affordable only"),
        raw.button("search", {
            frame: "button",
            transform: { section: "player", x: 6, y: 1, width: 2, height: 1 },
            tooltip: "Search the catalog",
            children: [raw.label("Find", text)],
        }),
        raw.button("clear_search", {
            transform: { section: "player", x: 8, y: 1, width: 1, height: 1 },
            states: enabledStates("Clear search", "clear_search"),
            children: [raw.spriteSlot("clear_search_icon", { width: 8, height: 6, sprite: "icon_clear" })],
        }),
        raw.slot("status", { ...text, x: 12, y: 179, width: 92, color: "#bceeff" }),
        raw.slot("balance", { ...text, x: 104, y: 179, width: 48, align: "right", color: "#ffb20b" }),
        raw.button("buy", {
            transform: { section: "hotbar", x: 0, y: 0, width: 6, height: 1 },
            states: enabledStates("Buy selected item", "buy"),
            children: [raw.slot("buy_label", { ...text, color: "#2a1200", shadow: false })],
        }),
        raw.button("exit", {
            frame: "button",
            transform: { section: "hotbar", x: 6, y: 0, width: 3, height: 1 },
            default: "close",
            tooltip: "Close market",
            children: [raw.label("Exit", text)],
        }),
        raw.slotRects("container_fill", {
            transform: { section: "container", x: 0, y: 0, width: 9, height: 6 },
            claim: "unowned",
        }),
        raw.slotRects("player_fill", {
            transform: { section: "player", x: 0, y: 0, width: 9, height: 3 },
            claim: "unowned",
        }),
        raw.slotRects("hotbar_fill", {
            transform: { section: "hotbar", x: 0, y: 0, width: 9, height: 1 },
            claim: "unowned",
        }),
    ],
});
