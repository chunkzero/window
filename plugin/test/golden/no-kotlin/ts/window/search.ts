import { anvilInput, button, label, panel, pattern, slotRects, sprite, ui } from "#plugins/window";

const text = { color: "#ffffff", shadow: true, small_caps: true };
const muted = { ...text, color: "#bceeff" };

/** A static anvil: nothing on it changes after open, so typing never reopens the screen. */
export default ui({
    name: "catalog_search",
    container: "anvil",
    bleed: { top: 1, right: 4, bottom: 7, left: 4 },
    children: [
        panel({ frame: "shell", x: -4, y: -1, width: 184, height: 174 }),
        panel({ frame: "recess", x: 56, y: 19, width: 112, height: 18 }),
        panel({ frame: "panel", x: 4, y: 72, width: 168, height: 88 }),
        panel({ frame: "hazard_bar", x: -4, y: 167, width: 184, height: 6 }),
        sprite("rivet", { x: -2, y: 6 }),
        sprite("rivet", { x: 173, y: 6 }),
        sprite("rivet", { x: -2, y: 157 }),
        sprite("rivet", { x: 173, y: 157 }),
        label("Search the Catalog", { ...text, x: 0, y: 6, width: 176, align: "center" }),
        label("Name", { ...muted, x: 8, y: 24, width: 44 }),
        anvilInput("query", { initial: "", item_model: "window:gui/hitbox" }),
        button("back", {
            frame: "button_danger",
            x: 58,
            y: 45,
            width: 52,
            height: 18,
            tooltip: "Back to market",
            children: [label("Back", text)],
        }),
        button("confirm", {
            frame: "button_confirm",
            x: 116,
            y: 45,
            width: 52,
            height: 18,
            tooltip: "Show matching products",
            children: [label("Search", text)],
        }),
        label("Matches names, tiers,", { ...muted, x: 8, y: 104, width: 160, align: "center" }),
        label("and categories", { ...muted, x: 8, y: 116, width: 160, align: "center" }),
        slotRects("inventory", {
            pattern: pattern.rect({ section: "player", x: 0, y: 0, width: 9, height: 3 }),
            claim: "all",
        }),
        slotRects("hotbar", {
            pattern: pattern.rect({ section: "hotbar", x: 0, y: 0, width: 9, height: 1 }),
            claim: "all",
        }),
    ],
});
