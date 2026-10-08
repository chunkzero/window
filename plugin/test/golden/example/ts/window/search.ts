import { action, input } from "plugin:window/bind";
import * as raw from "plugin:window/raw";
import { Button, art } from "plugin:window/theme/industrial";

import { icons } from "./art.ts";

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
        raw.box({ frame: art.shell, x: -4, y: -1, style: { width: 184, height: 80 } }),
        raw.box({ frame: art.recess, x: 30, y: 3, style: { width: 116, height: 12 } }),
        raw.box({ frame: art.panel, x: 4, y: 15, style: { width: 168, height: 54 } }),
        raw.box({ frame: art.hazardBar, x: -4, y: 73, style: { width: 184, height: 6 } }),
        raw.image(art.rivet, { x: -2, y: 6 }),
        raw.image(art.rivet, { x: 173, y: 6 }),
        raw.image(art.rivet, { x: -2, y: 68 }),
        raw.image(art.rivet, { x: 173, y: 68 }),
        raw.text("Catalog Search", { ...text, x: 0, y: 6, width: 176, align: "center" }),
        raw.text("Name", { ...muted, x: 10, y: 24, width: 44 }),
        raw.input(input("query"), { initial: "" }),
        raw.section("container", {
            claim: "none",
            children: [
                Button({ onClick: action("back"), at: [0, 0], children: raw.image(icons.back) }),
                Button({
                    onClick: action("confirm"),
                    at: [2, 0],
                    tooltip: "Search",
                    style: { base: { frame: art.buttonAccent } },
                    // A box one pixel wider than the icon centers it like the runtime does, rounding down.
                    children: raw.box({ style: { width: 10 }, children: [raw.image(icons.check)] }),
                }),
            ],
        }),
        raw.section("player", { claim: "all" }),
        raw.section("hotbar", { claim: "all" }),
    ],
});
