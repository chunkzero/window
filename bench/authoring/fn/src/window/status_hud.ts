import { raw } from "#plugins/window";
import type { Element, Hud, TextStyleOptions } from "#plugins/window";

const text: TextStyleOptions = { shadow: true, small_caps: true, color: "#ffffff" };
const muted = "#8fb3d9";
const soft = "#e0edff";

const ANCHORS = {
    "top-left": [0, 0],
    top: [0.5, 0],
    "top-right": [1, 0],
    left: [0, 0.5],
    right: [1, 0.5],
    bottom: [0.5, 1],
} as const;

/** A content-sized HUD pinned to `anchor`, its children in a column. */
function pinned(
    name: string,
    anchor: keyof typeof ANCHORS,
    offset: [number, number],
    opts: { frame?: string; padding?: number; gap?: number; minWidth?: number },
    children: Element[],
): { huds: Hud[] } {
    const [x, y] = ANCHORS[anchor];
    return raw.hud({
        name,
        ...(opts.frame === undefined ? {} : { frame: opts.frame }),
        shader: { origin: { x, y }, anchor: { x, y }, x: offset[0], y: offset[1] },
        children: [
            raw.flex({
                style: {
                    direction: "column",
                    ...(opts.gap === undefined ? {} : { gap: opts.gap }),
                    ...(opts.padding === undefined ? {} : { padding: opts.padding }),
                    ...(opts.minWidth === undefined ? {} : { min_width: opts.minWidth }),
                },
                children,
            }),
        ],
    });
}

/** A label and a right-aligned value; `coin` follows the value with a coin. */
function stat(title: string, bind: string, color?: string, coin = false): Element {
    return raw.flex({
        style: { direction: "row", align: "center", gap: 2 },
        children: [
            raw.label(title, { ...text, width: 34, color: muted }),
            raw.slot(bind, { ...text, align: "right", ...(color === undefined ? {} : { color }) }),
            ...(coin ? [raw.sprite("coin")] : []),
        ],
    });
}

export default [
    pinned("status_top_left", "top-left", [4, 4], { frame: "hud", padding: 4, gap: 2, minWidth: 80 }, [
        stat("Coins", "coins", "#ffd75e", true),
        stat("Rate", "rate", "#80ff80"),
        stat("Power", "power"),
    ]),
    pinned("status_top_center", "top", [0, 4], { frame: "hud", padding: 3 }, [
        raw.slot("runtime", { ...text, width: 34, align: "center" }),
    ]),
    pinned("status_top_right", "top-right", [-4, 4], { frame: "hud", padding: 4, gap: 2, minWidth: 80 }, [
        stat("Wave", "wave"),
        stat("Biome", "biome"),
        stat("Ping", "latency"),
    ]),
    pinned("status_left_side", "left", [4, 0], { gap: 2 }, [
        raw.slot("coords", { ...text, width: 64 }),
        raw.slot("altitude", { ...text, width: 64, color: soft }),
    ]),
    pinned("status_right_side", "right", [-4, 0], { gap: 2 }, [
        raw.slot("objective", { ...text, width: 80, align: "right", color: "#ffd75e" }),
        raw.slot("stock", { ...text, width: 80, align: "right", color: soft }),
    ]),
    pinned("status_bottom_center", "bottom", [0, -72], {}, [
        raw.slot("hint", { ...text, width: 160, align: "center", color: soft }),
    ]),
];
