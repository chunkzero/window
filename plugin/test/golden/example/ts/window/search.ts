import { pattern, raw } from "#plugins/window";

const text = { color: "#ffffff", shadow: true, small_caps: true };
const muted = { ...text, color: "#bceeff" };

/**
 * A static anvil: nothing on it changes after open, so typing never reopens the screen. Vanilla's anvil art is hidden,
 * and the player's inventory slots are claimed so the screen ends at the hazard bar.
 */
export default raw.ui({
    name: "catalog_search",
    container: "anvil",
    bleed: { top: 1, right: 4, bottom: 0, left: 4 },
    children: [
        raw.panel({ frame: "shell", x: -4, y: -1, width: 184, height: 80 }),
        raw.panel({ frame: "recess", x: 30, y: 3, width: 116, height: 12 }),
        raw.panel({ frame: "panel", x: 4, y: 15, width: 168, height: 54 }),
        raw.panel({ frame: "hazard_bar", x: -4, y: 73, width: 184, height: 6 }),
        raw.sprite("rivet", { x: -2, y: 6 }),
        raw.sprite("rivet", { x: 173, y: 6 }),
        raw.sprite("rivet", { x: -2, y: 68 }),
        raw.sprite("rivet", { x: 173, y: 68 }),
        raw.label("Catalog Search", { ...text, x: 0, y: 6, width: 176, align: "center" }),
        raw.label("Name", { ...muted, x: 10, y: 24, width: 44 }),
        raw.anvilInput("query", { initial: "" }),
        raw.button("back", {
            frame: "button",
            transform: { section: "container", x: 0, y: 0, width: 1, height: 1 },
            children: [raw.sprite("icon_back")],
        }),
        raw.button("confirm", {
            frame: "button_accent",
            transform: { section: "container", x: 2, y: 0, width: 1, height: 1 },
            tooltip: "Search",
            children: [raw.sprite("icon_check")],
        }),
        raw.slotRects("inventory", {
            pattern: pattern.rect({ section: "player", x: 0, y: 0, width: 9, height: 3 }),
            claim: "all",
        }),
        raw.slotRects("hotbar", {
            pattern: pattern.rect({ section: "hotbar", x: 0, y: 0, width: 9, height: 1 }),
            claim: "all",
        }),
    ],
});
