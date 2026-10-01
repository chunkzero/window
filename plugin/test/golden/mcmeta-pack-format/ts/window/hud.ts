import { hud, label, panel, slot } from "#plugins/window";

export default hud({
    name: "corner",
    channel: "actionbar",
    width: 80,
    height: 30,
    shader: { source_bottom: 59, origin: { x: 1.0, y: 0.0 }, anchor: { x: 1.0, y: 0.0 }, x: -4, y: 4 },
    children: [
        panel({ frame: "hud_panel", x: 0, y: 0, width: 80, height: 30 }),
        label("Score", { x: 6, y: 4, width: 40, color: "#ffd700" }),
        slot("score", { x: 46, y: 4, width: 28, align: "right", color: "#ffffff" }),
    ],
});
