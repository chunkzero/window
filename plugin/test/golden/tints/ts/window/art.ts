import { shape } from "plugin:window/ui";
import { tint } from "plugin:window/bind";

export const tone = tint("tone", { ok: "#3ddc84", warn: "#ffb020", danger: "#ff4d4d" });

/** White art, so a tint shows as its own color. */
export const pill = shape({
    kind: "panel",
    fill: "#ffffff",
    border_color: "#b0b0b0",
    border_width: 1,
    radius: 3,
    inset_depth: 0,
});

export const plate = shape({ kind: "panel", fill: "#20242c", border_width: 0, radius: 0, inset_depth: 0 });

export const dot = shape({ kind: "badge", width: 6, height: 6, fill: "#ffffff", border_width: 0, radius: 3 });

/** The runtime sprites Kotlin selects for sprite handles. */
export default { sprites: { gem: shape({ kind: "badge", width: 8, height: 8, fill: "#e0e0e0", radius: 2 }) } };
