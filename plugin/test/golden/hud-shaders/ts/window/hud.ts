import { text } from "plugin:window/bind";
import * as raw from "plugin:window/raw";
import { art } from "plugin:window/theme/industrial";

export default raw.hud({
    name: "corner",
    channel: "actionbar",
    width: 80,
    height: 30,
    shader: { source_bottom: 59, origin: { x: 1.0, y: 0.0 }, anchor: { x: 1.0, y: 0.0 }, x: -4, y: 4 },
    children: [
        raw.box({ frame: art.hud, x: 0, y: 0, style: { width: 80, height: 30 } }),
        raw.text("Score", { x: 6, y: 4, width: 40, color: "#ffd700" }),
        raw.text(text("score"), { x: 46, y: 4, width: 28, align: "right", color: "#ffffff" }),
    ],
});
