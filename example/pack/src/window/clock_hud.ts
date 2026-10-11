import { text } from "plugin:window/bind";
import * as raw from "plugin:window/raw";
import { art } from "plugin:window/theme/industrial";

export const runtime = text("runtime");

/** The clock, written with the function-style `raw` API; it builds the same document as a `<Hud>`. */
export default raw.hud({
    name: "status_top_center",
    frame: art.hud,
    shader: { origin: { x: 0.5, y: 0 }, anchor: { x: 0.5, y: 0 }, x: 0, y: 4 },
    children: [
        raw.box({
            style: { direction: "column", padding: 3 },
            children: [
                raw.text(runtime, { width: 34, align: "center", color: "#ffffff", shadow: true, small_caps: true }),
            ],
        }),
    ],
});
