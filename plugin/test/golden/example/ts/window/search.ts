import { anvilInput, button, collection, label, panel, pattern, slot, sprite, ui } from "#plugins/window";

const hitbox = "window:gui/hitbox";
const text = { color: "#ffffff", shadow: true, small_caps: true };
const muted = { ...text, color: "#bceeff" };

/** Text stays clear of the anvil's native slots at y 47..62, where vanilla draws its hover highlight. */
export default ui({
    name: "catalog_search",
    container: "anvil",
    bleed: { top: 1, right: 4, bottom: 7, left: 4 },
    children: [
        panel({ frame: "shell", x: -4, y: -1, width: 184, height: 174 }),
        panel({ frame: "panel", x: 4, y: 15, width: 168, height: 148 }),
        panel({ frame: "recess", x: 30, y: 3, width: 116, height: 12 }),
        panel({ frame: "recess", x: 56, y: 19, width: 112, height: 18 }),
        panel({ frame: "hazard_bar", x: -4, y: 167, width: 184, height: 6 }),
        sprite("rivet", { x: -2, y: 6 }),
        sprite("rivet", { x: 173, y: 6 }),
        sprite("rivet", { x: -2, y: 157 }),
        sprite("rivet", { x: 173, y: 157 }),
        label("Catalog Search", { ...text, x: 0, y: 6, width: 176, align: "center" }),
        label("Query", { ...muted, x: 8, y: 24, width: 44 }),
        anvilInput("query", { initial: "", item_model: hitbox }),
        label("Select a result to view it", { ...muted, x: 8, y: 40, width: 160 }),
        label("Results", { ...muted, x: 8, y: 68, width: 56 }),
        slot("result_count", { ...text, x: 64, y: 68, width: 104, align: "right", color: "#ffb20b" }),
        collection("results", {
            frame: "slot",
            pattern: pattern.rect({ section: "player", x: 0, y: 0, width: 9, height: 3 }),
        }),
        button("back", {
            frame: "button",
            transform: { section: "hotbar", x: 0, y: 0, width: 3, height: 1 },
            tooltip: "Back to market",
            children: [label("Back", text)],
        }),
        button("reset", {
            transform: { section: "hotbar", x: 3, y: 0, width: 3, height: 1 },
            states: {
                enabled: { item_model: hitbox, tooltip: "Clear the search", sprite: "action" },
                disabled: { item_model: hitbox, tooltip: "Nothing to clear", sprite: "action_disabled" },
            },
            children: [slot("reset_label", text)],
        }),
        button("exit", {
            frame: "button",
            transform: { section: "hotbar", x: 6, y: 0, width: 3, height: 1 },
            default: "close",
            tooltip: "Close search",
            children: [label("Close", text)],
        }),
    ],
});
