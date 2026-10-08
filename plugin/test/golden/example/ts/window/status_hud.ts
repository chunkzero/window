import { raw } from "#plugins/window";

const muted = "#8fb3d9";
const soft = "#e0edff";
const text = { shadow: true, small_caps: true };

/** A label and right-aligned value; `coin` reserves room for a coin after the value. */
function statRow(name: string, value: string, y: number, color = "#ffffff", coin = false) {
    return [
        raw.label(name, { ...text, x: 5, y, width: 36, color: muted }),
        raw.slot(value, { ...text, x: 40, y, width: coin ? 27 : 35, align: "right", color }),
        ...(coin ? [raw.sprite("coin", { x: 69, y })] : []),
    ];
}

export default {
    huds: [
        {
            name: "status_top_left",
            channel: "actionbar",
            width: 80,
            height: 35,
            shader: { source_bottom: 59, origin: { x: 0.0, y: 0.0 }, anchor: { x: 0.0, y: 0.0 }, x: 4, y: 4 },
            children: [
                raw.panel({ frame: "hud", x: 0, y: 0, width: 80, height: 35 }),
                ...statRow("Coins", "coins", 4, "#ffd75e", true),
                ...statRow("Rate", "rate", 14, "#80ff80"),
                ...statRow("Power", "power", 24),
            ],
        },
        {
            name: "status_top_center",
            channel: "actionbar",
            width: 40,
            height: 14,
            shader: { source_bottom: 59, origin: { x: 0.5, y: 0.0 }, anchor: { x: 0.5, y: 0.0 }, x: 0, y: 4 },
            children: [
                raw.panel({ frame: "hud", x: 0, y: 0, width: 40, height: 14 }),
                raw.slot("runtime", { ...text, x: 2, y: 3, width: 36, align: "center", color: "#ffffff" }),
            ],
        },
        {
            name: "status_top_right",
            channel: "actionbar",
            width: 80,
            height: 35,
            shader: { source_bottom: 59, origin: { x: 1.0, y: 0.0 }, anchor: { x: 1.0, y: 0.0 }, x: -4, y: 4 },
            children: [
                raw.panel({ frame: "hud", x: 0, y: 0, width: 80, height: 35 }),
                ...statRow("Wave", "wave", 4),
                ...statRow("Biome", "biome", 14),
                ...statRow("Ping", "latency", 24),
            ],
        },
        {
            name: "status_left_side",
            channel: "actionbar",
            width: 64,
            height: 18,
            shader: { source_bottom: 59, origin: { x: 0.0, y: 0.5 }, anchor: { x: 0.0, y: 0.5 }, x: 4, y: 0 },
            children: [
                raw.slot("coords", { ...text, x: 0, y: 0, width: 64, color: "#ffffff" }),
                raw.slot("altitude", { ...text, x: 0, y: 10, width: 64, color: soft }),
            ],
        },
        {
            name: "status_right_side",
            channel: "actionbar",
            width: 80,
            height: 18,
            shader: { source_bottom: 59, origin: { x: 1.0, y: 0.5 }, anchor: { x: 1.0, y: 0.5 }, x: -4, y: 0 },
            children: [
                raw.slot("objective", { ...text, x: 0, y: 0, width: 80, align: "right", color: "#ffd75e" }),
                raw.slot("stock", { ...text, x: 0, y: 10, width: 80, align: "right", color: soft }),
            ],
        },
        {
            name: "status_bottom_center",
            channel: "actionbar",
            width: 160,
            height: 8,
            shader: { source_bottom: 59, origin: { x: 0.5, y: 1.0 }, anchor: { x: 0.5, y: 1.0 }, x: 0, y: -72 },
            children: [raw.slot("hint", { ...text, x: 0, y: 0, width: 160, align: "center", color: soft })],
        },
    ],
};
