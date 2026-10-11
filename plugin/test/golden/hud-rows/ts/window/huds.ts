import { text } from "plugin:window/bind";
import * as raw from "plugin:window/raw";
import { art } from "plugin:window/theme/industrial";

/** A label and a right-aligned value on row `y`. */
function stat(name: string, y: number) {
    return [
        raw.text(name, { x: 6, y, width: 40, color: "#ffd700" }),
        raw.text(text(name.toLowerCase()), { x: 46, y, width: 28, align: "right", color: "#ffffff" }),
    ];
}

function pin(x: number, y: number, dx: number, dy: number) {
    return { source_bottom: 59, origin: { x, y }, anchor: { x, y }, x: dx, y: dy };
}

// Shader-placed HUDs whose text rows do not line up, and a fallback HUD that keeps its own row.
export default {
    huds: [
        {
            name: "alpha",
            channel: "actionbar",
            width: 80,
            height: 30,
            shader: pin(0, 0, 4, 4),
            children: [
                raw.box({ frame: art.hud, x: 0, y: 0, style: { width: 80, height: 30 } }),
                ...stat("Coins", 4),
                ...stat("Rate", 16),
            ],
        },
        {
            name: "beta",
            channel: "actionbar",
            width: 80,
            height: 45,
            shader: pin(1, 0, -4, 4),
            children: [
                raw.box({ frame: art.hud, x: 0, y: 0, style: { width: 80, height: 45 } }),
                ...stat("Wave", 7),
                ...stat("Biome", 19),
                ...stat("Ping", 31),
            ],
        },
        {
            name: "delta",
            channel: "actionbar",
            width: 80,
            height: 22,
            shader: pin(0, 0.5, 4, 0),
            children: [raw.text(text("objective"), { x: 0, y: 2, width: 80, lines: 2, line_height: 10 })],
        },
        {
            name: "gamma",
            channel: "actionbar",
            width: 80,
            height: 17,
            shader: pin(0.5, 1, 0, -72),
            children: [...stat("Hint", 0), ...stat("Next", 9)],
        },
        {
            name: "plain",
            channel: "bossbar",
            width: 80,
            height: 13,
            children: [...stat("Boss", 5)],
        },
    ],
};
