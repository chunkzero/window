import { action, flag, industrial, items, raw, sprite, text, toggle, value } from "#plugins/window";
import type { Element, LabelOptions, SlotOptions, TextHandle } from "#plugins/window";

import { coin } from "./sprites.ts";

const style = { color: "#ffffff", shadow: true, small_caps: true };

const say = (value: string, opts: LabelOptions = {}) => raw.label(value, { ...style, ...opts });
const bound = (handle: TextHandle, opts: SlotOptions = {}) => raw.slot(handle, { ...style, ...opts });

const mode = value("mode", ["idle", "forging"]);
const progress = text("progress");
const recipe = action("recipe", { shape: [3] });
const stack = items("stack", { shape: [3] });
const recipeName = text("recipe_name", { shape: [3] });
const cost = text("cost", { shape: [3] });
const forgeIcon = sprite("forge_icon");
const autoLamp = sprite("auto_lamp", { only: ["lamp_on", "lamp_off"] });

function meter(title: string, handle: TextHandle, color: string): Element {
    return industrial.Row({
        gap: 2,
        justify: "center",
        children: [say(title, { color: "#8fb3d9" }), bound(handle, { width: 18, color })],
    });
}

const forge = raw.ui({
    name: "forge",
    container: "generic_9x4",
    frame: industrial.art.shell,
    bleed: { top: 1, right: 4, left: 4 },
    children: [
        industrial.Header({
            padding: { top: 3 },
            align: "start",
            children: industrial.Row({
                frame: industrial.art.recess,
                height: 12,
                padding: { left: 4, right: 4 },
                gap: 3,
                children: [
                    raw.image(coin, { layout: { translate: [0, 1] } }),
                    say("Rune Forge"),
                    raw.image(coin, { layout: { translate: [0, 1] } }),
                ],
            }),
        }),
        raw.switchOn(
            mode,
            [
                raw.case("idle", {
                    frame: industrial.art.buttonDanger,
                    style: { width: 24, height: 12, justify: "center", align: "center" },
                    children: [say("New")],
                }),
                raw.case("forging", {
                    frame: industrial.art.buttonAccent,
                    style: { justify: "center", align: "center" },
                    children: [bound(progress, { width: 20, align: "center", color: "#2a1200", shadow: false })],
                }),
            ],
            { x: 146, y: 3 },
        ),

        raw.section("container", {
            frame: industrial.art.panel,
            outset: { top: 2, right: 3, bottom: 3, left: 3 },
            children: [
                industrial.Repeater({
                    onClick: recipe,
                    item: stack,
                    cell: [3, 2],
                    columns: 3,
                    rows: 1,
                    children: (i) => [
                        bound(recipeName.at(i), { width: 50, align: "center" }),
                        industrial.Row({
                            gap: 1,
                            children: [
                                bound(cost.at(i), { width: 20, align: "right", color: "#ffb20b" }),
                                raw.image(coin),
                            ],
                        }),
                    ],
                }),

                industrial.Grid({
                    span: 9,
                    columns: 3,
                    frame: industrial.art.recess,
                    align: "center",
                    children: [
                        meter("Heat", text("heat"), "#ff8300"),
                        meter("Ore", text("ore"), "#bceeff"),
                        meter("Time", text("time"), "#80ff80"),
                    ],
                }),

                industrial.Hotspot({ tooltip: { title: "Forging", lines: ["Pick a recipe", "Then press Forge"] } }),
                industrial.Spacer({ span: 4 }),
                industrial.Button({
                    onClick: action("forge"),
                    span: 4,
                    frame: industrial.art.buttonAccent,
                    tooltip: "Forge the selected recipe",
                    children: [
                        raw.image(forgeIcon, { width: 8, height: 8 }),
                        bound(text("forge_label"), { color: "#2a1200", shadow: false }),
                    ],
                }),
            ],
        }),

        raw.section("player", {
            children: [
                industrial.Slots({ span: [9, 2] }),
                industrial.Row({
                    span: 6,
                    frame: industrial.art.recess,
                    padding: { left: 4, right: 4 },
                    children: [bound(text("status"), { color: "#bceeff" })],
                }),
                industrial.Toggle({
                    bind: toggle("auto"),
                    span: 3,
                    on: { tooltip: "Auto forge on" },
                    off: { tooltip: "Auto forge off" },
                    children: [raw.image(autoLamp, { width: 4, height: 4 }), say("Auto")],
                }),
            ],
        }),

        raw.section("hotbar", {
            children: [
                industrial.Button({
                    close: true,
                    at: [3, 0],
                    span: 3,
                    tooltip: "Leave the forge",
                    children: [say("Leave")],
                }),
            ],
        }),
    ],
});

const progressHud = raw.hud({
    name: "forge_progress",
    frame: industrial.art.hud,
    shader: { origin: { x: 0.5, y: 1 }, anchor: { x: 0.5, y: 1 }, x: 0, y: -60 },
    children: [
        raw.box({
            style: { direction: "column", padding: 3, gap: 2, minWidth: 90 },
            children: [
                industrial.Row({
                    gap: 4,
                    children: [
                        say("Forging", { layout: { grow: 1 } }),
                        industrial.Show({
                            when: flag("overheated"),
                            children: industrial.Row({
                                frame: industrial.art.recess,
                                padding: { left: 2, right: 2 },
                                gap: 2,
                                children: [raw.image(coin), say("Hot", { color: "#ff8300" })],
                            }),
                        }),
                        bound(text("percent"), { width: 24, align: "right", color: "#ffd75e" }),
                    ],
                }),
                industrial.Grid({
                    columns: ["1fr", "1fr"],
                    gap: [4, 1],
                    children: [
                        bound(text("item"), { color: "#e0edff" }),
                        bound(text("eta"), { align: "right", color: "#8fb3d9" }),
                    ],
                }),
            ],
        }),
    ],
});

export default [forge, progressHud];
