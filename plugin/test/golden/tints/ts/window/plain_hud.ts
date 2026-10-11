import { text } from "plugin:window/bind";
import * as raw from "plugin:window/raw";

import { dot, pill, tone } from "./art.ts";

// A HUD without a shader, written with the function-style API.
export default raw.hud({
    name: "plain",
    channel: "bossbar",
    width: 80,
    height: 13,
    children: [
        raw.box({ frame: pill, tint: tone, x: 0, y: 0, style: { width: 80, height: 13 } }),
        raw.image(dot, { x: 3, y: 3, tint: "#ff0000" }),
        raw.text(text("boss"), { x: 12, y: 3, width: 60, color: tone }),
    ],
});
