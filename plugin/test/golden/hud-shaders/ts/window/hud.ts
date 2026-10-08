import { raw } from "#plugins/window";

export default raw.hud({
    name: "corner",
    channel: "actionbar",
    width: 80,
    height: 30,
    shader: { source_bottom: 59, origin: { x: 1.0, y: 0.0 }, anchor: { x: 1.0, y: 0.0 }, x: -4, y: 4 },
    children: [
        raw.panel({ frame: "hud", x: 0, y: 0, width: 80, height: 30 }),
        raw.label("Score", { x: 6, y: 4, width: 40, color: "#ffd700" }),
        raw.slot("score", { x: 46, y: 4, width: 28, align: "right", color: "#ffffff" }),
    ],
});
