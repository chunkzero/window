import {
    button,
    choice,
    collection,
    label,
    panel,
    pattern,
    slot,
    slotRects,
    sprite,
    spriteSlot,
    toggle,
    ui,
} from "#plugins/window";

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
    return toggle(name, {
        frame: "button",
        transform: { section: "player", x, y: 1, width: 3, height: 1 },
        states: {
            on: { item_model: hitbox, tooltip: tooltip + " on" },
            off: { item_model: hitbox, tooltip: tooltip + " off" },
        },
        children: [
            spriteSlot(name + "_lamp", { x: 5, y: 6, width: 4, height: 4 }),
            label(title, { ...text, x: 12, y: 4, width: 36, align: "center" }),
        ],
    });
}
export default ui({
    name: "shop",
    container: "generic_9x6",
    bleed: { top: 1, right: 4, bottom: 7, left: 4 },
    children: [
        panel({ frame: "shell", x: -4, y: -1, width: 184, height: 230 }),
        panel({ frame: "panel", x: 4, y: 15, width: 168, height: 113 }),
        panel({ frame: "panel", x: 4, y: 136, width: 168, height: 82 }),
        panel({ frame: "recess", x: 30, y: 3, width: 116, height: 12 }),
        panel({ frame: "recess", x: 62, y: 90, width: 52, height: 16 }),
        panel({ frame: "recess", x: 8, y: 108, width: 160, height: 16 }),
        panel({ frame: "recess", x: 8, y: 175, width: 160, height: 16 }),
        panel({ frame: "hazard_bar", x: -4, y: 223, width: 184, height: 6 }),
        ...[52, 62, 72, 82, 92, 102, 112, 122].map((x) => sprite("vent_slot", { x, y: 131 })),
        sprite("rivet", { x: -2, y: 6 }),
        sprite("rivet", { x: 173, y: 6 }),
        sprite("rivet", { x: -2, y: 129 }),
        sprite("rivet", { x: 173, y: 129 }),
        sprite("coin", { x: 156, y: 179 }),
        label("Foundry Exchange", { ...text, x: 0, y: 6, width: 176, align: "center" }),
        choice("category_all", {
            transform: { section: "container", x: 0, y: 0, width: 3, height: 1 },
            states: choiceStates("All items"),
            children: [slot("category_all_label", text)],
        }),
        choice("category_gear", {
            transform: { section: "container", x: 3, y: 0, width: 3, height: 1 },
            states: choiceStates("Gear"),
            children: [slot("category_gear_label", text)],
        }),
        choice("category_magic", {
            transform: { section: "container", x: 6, y: 0, width: 3, height: 1 },
            states: choiceStates("Magic"),
            children: [slot("category_magic_label", text)],
        }),
        collection("products", {
            frame: "slot",
            selected_sprite: "slot_selected",
            pattern: pattern.rect({ section: "container", x: 0, y: 1, width: 9, height: 3 }),
        }),
        button("previous", {
            transform: { section: "container", x: 0, y: 4, width: 3, height: 1 },
            states: enabledStates("Previous page", "action"),
            children: [slot("previous_label", text)],
        }),
        slot("page", { ...text, x: 62, y: 94, width: 52, align: "center" }),
        button("next", {
            transform: { section: "container", x: 6, y: 4, width: 3, height: 1 },
            states: enabledStates("Next page", "action"),
            children: [slot("next_label", text)],
        }),
        slot("selection", { ...text, x: 12, y: 112, width: 100, color: "#ffb20b" }),
        slot("price", { ...text, x: 112, y: 112, width: 40, align: "right" }),
        spriteSlot("price_coin", { x: 156, y: 112, width: 8, height: 8 }),
        choice("sort_featured", {
            transform: { section: "player", x: 0, y: 0, width: 3, height: 1 },
            states: choiceStates("Top picks first"),
            children: [slot("sort_featured_label", text)],
        }),
        choice("sort_price", {
            transform: { section: "player", x: 3, y: 0, width: 3, height: 1 },
            states: choiceStates("Price sort"),
            children: [slot("sort_price_label", text)],
        }),
        choice("sort_name", {
            transform: { section: "player", x: 6, y: 0, width: 3, height: 1 },
            states: choiceStates("Name sort"),
            children: [slot("sort_name_label", text)],
        }),
        lampToggle("favorites", 0, "Favs", "Favorites only"),
        lampToggle("affordable", 3, "Afford", "Affordable only"),
        button("search", {
            frame: "button",
            transform: { section: "player", x: 6, y: 1, width: 3, height: 1 },
            tooltip: "Search the catalog",
            children: [label("Search", text)],
        }),
        slot("status", { ...text, x: 12, y: 179, width: 92, color: "#bceeff" }),
        slot("balance", { ...text, x: 104, y: 179, width: 48, align: "right", color: "#ffb20b" }),
        button("buy", {
            transform: { section: "hotbar", x: 0, y: 0, width: 6, height: 1 },
            states: enabledStates("Buy selected item", "buy"),
            children: [slot("buy_label", { ...text, color: "#2a1200", shadow: false })],
        }),
        button("exit", {
            frame: "button",
            transform: { section: "hotbar", x: 6, y: 0, width: 3, height: 1 },
            default: "close",
            tooltip: "Close market",
            children: [label("Exit", text)],
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
