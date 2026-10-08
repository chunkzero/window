import { texture } from "plugin:window/ui";
import type { WindowDocument } from "plugin:window/ui";
import { sprite } from "plugin:window/bind";
import * as raw from "plugin:window/raw";

const frame = texture("window/frame.png", { insets: 3 });
const badge = texture("window:gui/badge.png", { width: 12, height: 12 });
const extra = texture("window:gui/extra", { width: 10, height: 10 });

/** Resource-pack textures stretched over boxes of the catalog sizes, drawn into the window's static art. */
const textured = raw.ui({
    name: "textured",
    container: "generic_9x1",
    children: [
        raw.box({ frame, x: 0, y: 0, style: { width: 100, height: 40 } }),
        raw.box({ frame: badge, x: 4, y: 12, style: { width: 12, height: 12 } }),
        raw.box({ frame: extra, x: 20, y: 12, style: { width: 10, height: 10 } }),
    ],
});

/** The same textures as catalog sprites, which the view picks at runtime. */
const picked = raw.ui({
    name: "picked",
    container: "generic_9x1",
    children: [
        raw.box({ frame, x: 0, y: 0, style: { width: 100, height: 40 } }),
        raw.image(sprite("badge"), { x: 4, y: 12, width: 12, height: 12 }),
        raw.image(sprite("extra"), { x: 20, y: 12, width: 10, height: 10 }),
    ],
});

const document: WindowDocument = {
    sprites: { badge, extra },
    windows: [...textured.windows, ...picked.windows],
};

export default document;
