import { pattern, raw } from "#plugins/window";
import type { Element, LabelOptions, SlotOptions } from "#plugins/window";

const text = { color: "#ffffff", shadow: true, small_caps: true };
const span = (columns: number) => ({ column: { span: columns } });

const say = (value: string, opts: LabelOptions = {}) => raw.label(value, { ...text, ...opts });
const bound = (name: string, opts: SlotOptions = {}) => raw.slot(name, { ...text, ...opts });

/** Button content: a centered row with centered text. */
function content(...children: Element[]): Element[] {
    return [raw.flex({ style: { direction: "row", justify: "center", align: "center", gap: 2 }, children })];
}

function meter(title: string, bind: string, color: string): Element {
    return raw.flex({
        style: { direction: "row", justify: "center", align: "center", gap: 2 },
        children: [say(title, { color: "#8fb3d9" }), bound(bind, { width: 18, color })],
    });
}

const forge = raw.ui({
    name: "forge",
    container: "generic_9x4",
    frame: "shell",
    bleed: { top: 1, right: 4, left: 4 },
    children: [
        raw.flex({
            x: 0,
            y: 0,
            style: { justify: "center", align: "start", gap: 4, padding: { top: 3 }, width: 176, height: 17 },
            children: [
                raw.flex({
                    frame: "recess",
                    style: { direction: "row", align: "center", gap: 3, padding: { left: 4, right: 4 }, height: 12 },
                    children: [
                        raw.sprite("coin", { layout: { translate: [0, 1] } }),
                        say("Rune Forge"),
                        raw.sprite("coin", { layout: { translate: [0, 1] } }),
                    ],
                }),
            ],
        }),
        raw.switchOn(
            "mode",
            {
                idle: {
                    frame: "button_danger",
                    style: { justify: "center", align: "center", width: 24, height: 12 },
                    children: [say("New")],
                },
                forging: {
                    frame: "button_accent",
                    style: { justify: "center", align: "center" },
                    children: [bound("progress", { width: 20, align: "center", color: "#2a1200", shadow: false })],
                },
            },
            { x: 146, y: 3 },
        ),

        raw.section("container", {
            frame: "panel",
            outset: { top: 2, right: 3, bottom: 3, left: 3 },
            children: [
                raw.repeater("recipe", {
                    frame: "button",
                    pattern: pattern.grid({ x: 0, y: 0, columns: 3, rows: 1, cell_width: 3, cell_height: 2 }),
                    children: [
                        raw.item("stack", { cell_slot: 1 }),
                        raw.flex({
                            style: { direction: "column", justify: "center", align: "center" },
                            children: [
                                bound("name", { width: 50, align: "center" }),
                                raw.flex({
                                    style: { direction: "row", align: "center", gap: 1 },
                                    children: [
                                        bound("cost", { width: 20, align: "right", color: "#ffb20b" }),
                                        raw.sprite("coin"),
                                    ],
                                }),
                            ],
                        }),
                    ],
                }),

                raw.grid({
                    frame: "recess",
                    style: { align: "center", columns: 3 },
                    layout: span(9),
                    children: [
                        meter("Heat", "heat", "#ff8300"),
                        meter("Ore", "ore", "#bceeff"),
                        meter("Time", "time", "#80ff80"),
                    ],
                }),

                raw.hotspot("help", { tooltip: { title: "Forging", lines: ["Pick a recipe", "Then press Forge"] } }),
                raw.flex({ layout: { grow: 1, ...span(4) } }),
                raw.button("forge", {
                    frame: "button_accent",
                    tooltip: "Forge the selected recipe",
                    layout: span(4),
                    children: content(
                        raw.spriteSlot("forge_icon", { width: 8, height: 8 }),
                        bound("forge_label", { align: "center", color: "#2a1200", shadow: false }),
                    ),
                }),
            ],
        }),

        raw.section("player", {
            children: [
                raw.slotRects("materials", {
                    frame: "slot",
                    claim: "none",
                    pattern: pattern.rect({ x: 0, y: 0, width: 9, height: 2 }),
                }),
                raw.flex({
                    frame: "recess",
                    style: { direction: "row", align: "center", padding: { left: 4, right: 4 } },
                    layout: span(6),
                    children: [bound("status", { color: "#bceeff" })],
                }),
                raw.toggle("auto", {
                    frame: "button",
                    states: { on: { tooltip: "Auto forge on" }, off: { tooltip: "Auto forge off" } },
                    layout: span(3),
                    children: content(
                        raw.spriteSlot("auto_lamp", { width: 4, height: 4 }),
                        say("Auto", { align: "center" }),
                    ),
                }),
            ],
        }),

        raw.section("hotbar", {
            children: [
                raw.button("leave", {
                    frame: "button",
                    default: "close",
                    tooltip: "Leave the forge",
                    layout: { column: { start: 4, span: 3 }, row: 1 },
                    children: content(say("Leave", { align: "center" })),
                }),
            ],
        }),
    ],
});

const progress = raw.hud({
    name: "forge_progress",
    frame: "hud",
    shader: { origin: { x: 0.5, y: 1 }, anchor: { x: 0.5, y: 1 }, x: 0, y: -60 },
    children: [
        raw.flex({
            style: { direction: "column", gap: 2, padding: 3, min_width: 90 },
            children: [
                raw.flex({
                    style: { direction: "row", align: "center", gap: 4 },
                    children: [
                        say("Forging", { layout: { grow: 1 } }),
                        raw.show("overheated", {
                            children: [
                                raw.flex({
                                    frame: "recess",
                                    style: {
                                        direction: "row",
                                        align: "center",
                                        gap: 2,
                                        padding: { left: 2, right: 2 },
                                    },
                                    children: [raw.sprite("coin"), say("Hot", { color: "#ff8300" })],
                                }),
                            ],
                        }),
                        bound("percent", { width: 24, align: "right", color: "#ffd75e" }),
                    ],
                }),
                raw.grid({
                    style: { gap: [4, 1], columns: ["1fr", "1fr"] },
                    children: [bound("item", { color: "#e0edff" }), bound("eta", { align: "right", color: "#8fb3d9" })],
                }),
            ],
        }),
    ],
});

export default [forge, progress];
