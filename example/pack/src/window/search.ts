import { anvilInput, button, label, panel, pattern, slotRects, sprite, ui } from "#plugins/window";

const text = { color: "#ffffff", shadow: true, small_caps: true };

/**
 * A static anvil: nothing on it changes after open, so typing never reopens the screen. Vanilla's anvil art is hidden,
 * and the player's inventory slots are claimed so the screen ends at the hazard bar.
 */
export default ui({
    name: "catalog_search",
    container: "anvil",
    children: [
        panel({ frame: "panel", x: 0, y: 0, width: 176, height: 17 }),
        panel({ frame: "hazard_bar", x: 0, y: 0, width: 10, height: 17 }),
        panel({ frame: "hazard_bar", x: 166, y: 0, width: 10, height: 17 }),
        label("Search the Catalog", { ...text, x: 0, y: 5, width: 176, align: "center" }),
        panel({ frame: "shell", x: 0, y: 16, width: 176, height: 52 }),
        panel({ frame: "panel", x: 36, y: 18, width: 137, height: 20 }),
        panel({ frame: "slot", x: 39, y: 19, width: 18, height: 18 }),
        sprite("icon_search", { x: 43, y: 23 }),
        anvilInput("query", { initial: "" }),
        button("back", {
            frame: "button_danger",
            transform: { section: "container", x: 0, y: 0, width: 1, height: 1 },
            children: [sprite("icon_back")],
        }),
        button("confirm", {
            frame: "button_confirm",
            transform: { section: "container", x: 2, y: 0, width: 1, height: 1 },
            tooltip: "Search",
            children: [sprite("icon_check")],
        }),
        panel({ frame: "hazard_bar", x: 0, y: 68, width: 176, height: 6 }),
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
