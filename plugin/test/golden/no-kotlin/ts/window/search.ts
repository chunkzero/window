import { anvilInput, button, collection, label, panel, pattern, slot, sprite, ui } from "#plugins/window";

const hitbox = "window:gui/hitbox";
export default ui({
    name: "catalog_search",
    container: "anvil",
    bleed: { top: 24, right: 40, bottom: 40, left: 40 },
    children: [
        panel({ frame: "shell", x: -38, y: -18, width: 252, height: 221 }),
        panel({ frame: "rail", x: -34, y: -14, width: 12, height: 207 }),
        panel({ frame: "rail", x: 198, y: -14, width: 12, height: 207 }),
        panel({ frame: "surface", x: -22, y: -14, width: 220, height: 58 }),
        panel({ frame: "header", x: -22, y: 44, width: 220, height: 19 }),
        panel({ frame: "panel", x: -22, y: 63, width: 220, height: 107 }),
        panel({ frame: "recess", x: 4, y: 84, width: 168, height: 52 }),
        panel({ frame: "vent", x: -14, y: 174, width: 74, height: 13 }),
        sprite("pack_badge", { x: 74, y: 173 }),
        panel({ frame: "vent", x: 116, y: 174, width: 74, height: 13 }),
        panel({ frame: "hazard_bar", x: -30, y: 191, width: 236, height: 10 }),
        panel({ frame: "hazard_bar", x: -14, y: 16, width: 14, height: 25 }),
        panel({ frame: "vent", x: 6, y: 18, width: 40, height: 21 }),
        panel({ frame: "hazard_bar", x: 176, y: 16, width: 14, height: 25 }),
        label("SEARCH", { x: 56, y: 8, width: 114, align: "center", color: "#bceeff", bold: true }),
        label("CATALOG SEARCH", { x: 0, y: 49, width: 105, align: "center", color: "#ffffff", bold: true }),
        panel({ frame: "search_field", x: 56, y: 19, width: 114, height: 18 }),
        slot("query_text", { x: 62, y: 24, width: 103, color: "#ffffff" }),
        anvilInput("query", { initial: "", item_model: hitbox }),
        slot("result_count", { x: 112, y: 49, width: 64, align: "right", color: "#bceeff" }),
        collection("results", {
            frame: "slot_cell",
            pattern: pattern.rect({ section: "player", x: 0, y: 0, width: 9, height: 3 }),
        }),
        button("back", {
            frame: "button_danger",
            transform: { section: "hotbar", x: 0, y: 0, width: 1, height: 1 },
            tooltip: "Back to market",
            children: [label("<", { color: "#ffffff", bold: true })],
        }),
        button("reset", {
            frame: "button_accent",
            transform: { section: "hotbar", x: 4, y: 0, width: 1, height: 1 },
            tooltip: "Clear the current search",
            children: [label("C", { color: "#1d2038", bold: true })],
        }),
        button("exit", {
            frame: "button_confirm",
            transform: { section: "hotbar", x: 8, y: 0, width: 1, height: 1 },
            default: "close",
            tooltip: "Close search",
            children: [label("OK", { color: "#ffffff", bold: true })],
        }),
    ],
});
