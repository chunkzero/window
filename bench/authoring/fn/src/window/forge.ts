import {
    button,
    flex,
    grid,
    hotspot,
    hud,
    item,
    label,
    pattern,
    repeater,
    section,
    show,
    slot,
    slotRects,
    sprite,
    spriteSlot,
    switchOn,
    toggle,
    ui,
} from "#plugins/window";
import type { Element, LabelOptions, SlotOptions } from "#plugins/window";

const text = { color: "#ffffff", shadow: true, small_caps: true };
const span = (columns: number) => ({ column: { span: columns } });

const say = (value: string, opts: LabelOptions = {}) => label(value, { ...text, ...opts });
const bound = (name: string, opts: SlotOptions = {}) => slot(name, { ...text, ...opts });

/** Button content: a centered row with centered text. */
function content(...children: Element[]): Element[] {
    return [flex({ style: { direction: "row", justify: "center", align: "center", gap: 2 }, children })];
}

function meter(title: string, bind: string, color: string): Element {
    return flex({
        style: { direction: "row", justify: "center", align: "center", gap: 2 },
        children: [say(title, { color: "#8fb3d9" }), bound(bind, { width: 18, color })],
    });
}

const forge = ui({
    name: "forge",
    container: "generic_9x4",
    frame: "shell",
    bleed: { top: 1, right: 4, left: 4 },
    children: [
        flex({
            x: 0,
            y: 0,
            style: { justify: "center", align: "start", gap: 4, padding: { top: 3 }, width: 176, height: 17 },
            children: [
                flex({
                    frame: "recess",
                    style: { direction: "row", align: "center", gap: 3, padding: { left: 4, right: 4 }, height: 12 },
                    children: [
                        sprite("coin", { layout: { translate: [0, 1] } }),
                        say("Rune Forge"),
                        sprite("coin", { layout: { translate: [0, 1] } }),
                    ],
                }),
            ],
        }),
        switchOn(
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

        section("container", {
            frame: "panel",
            outset: { top: 2, right: 3, bottom: 3, left: 3 },
            children: [
                repeater("recipe", {
                    frame: "button",
                    pattern: pattern.grid({ x: 0, y: 0, columns: 3, rows: 1, cell_width: 3, cell_height: 2 }),
                    children: [
                        item("stack", { cell_slot: 1 }),
                        flex({
                            style: { direction: "column", justify: "center", align: "center" },
                            children: [
                                bound("name", { width: 50, align: "center" }),
                                flex({
                                    style: { direction: "row", align: "center", gap: 1 },
                                    children: [
                                        bound("cost", { width: 20, align: "right", color: "#ffb20b" }),
                                        sprite("coin"),
                                    ],
                                }),
                            ],
                        }),
                    ],
                }),

                grid({
                    frame: "recess",
                    style: { align: "center", columns: 3 },
                    layout: span(9),
                    children: [
                        meter("Heat", "heat", "#ff8300"),
                        meter("Ore", "ore", "#bceeff"),
                        meter("Time", "time", "#80ff80"),
                    ],
                }),

                hotspot("help", { tooltip: { title: "Forging", lines: ["Pick a recipe", "Then press Forge"] } }),
                flex({ layout: { grow: 1, ...span(4) } }),
                button("forge", {
                    frame: "button_accent",
                    tooltip: "Forge the selected recipe",
                    layout: span(4),
                    children: content(
                        spriteSlot("forge_icon", { width: 8, height: 8 }),
                        bound("forge_label", { align: "center", color: "#2a1200", shadow: false }),
                    ),
                }),
            ],
        }),

        section("player", {
            children: [
                slotRects("materials", {
                    frame: "slot",
                    claim: "none",
                    pattern: pattern.rect({ x: 0, y: 0, width: 9, height: 2 }),
                }),
                flex({
                    frame: "recess",
                    style: { direction: "row", align: "center", padding: { left: 4, right: 4 } },
                    layout: span(6),
                    children: [bound("status", { color: "#bceeff" })],
                }),
                toggle("auto", {
                    frame: "button",
                    states: { on: { tooltip: "Auto forge on" }, off: { tooltip: "Auto forge off" } },
                    layout: span(3),
                    children: content(
                        spriteSlot("auto_lamp", { width: 4, height: 4 }),
                        say("Auto", { align: "center" }),
                    ),
                }),
            ],
        }),

        section("hotbar", {
            children: [
                button("leave", {
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

const progress = hud({
    name: "forge_progress",
    frame: "hud",
    shader: { origin: { x: 0.5, y: 1 }, anchor: { x: 0.5, y: 1 }, x: 0, y: -60 },
    children: [
        flex({
            style: { direction: "column", gap: 2, padding: 3, min_width: 90 },
            children: [
                flex({
                    style: { direction: "row", align: "center", gap: 4 },
                    children: [
                        say("Forging", { layout: { grow: 1 } }),
                        show("overheated", {
                            children: [
                                flex({
                                    frame: "recess",
                                    style: {
                                        direction: "row",
                                        align: "center",
                                        gap: 2,
                                        padding: { left: 2, right: 2 },
                                    },
                                    children: [sprite("coin"), say("Hot", { color: "#ff8300" })],
                                }),
                            ],
                        }),
                        bound("percent", { width: 24, align: "right", color: "#ffd75e" }),
                    ],
                }),
                grid({
                    style: { gap: [4, 1], columns: ["1fr", "1fr"] },
                    children: [bound("item", { color: "#e0edff" }), bound("eta", { align: "right", color: "#8fb3d9" })],
                }),
            ],
        }),
    ],
});

export default [forge, progress];
