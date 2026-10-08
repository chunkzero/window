import { raw, sprite, texture } from "#plugins/window";
import type { WindowDocument } from "#plugins/window";

const textured = raw.ui({
    name: "textured",
    container: "generic_9x1",
    children: [
        raw.box({ frame: texture("window/frame.png", { insets: 3 }), x: 0, y: 0, style: { width: 100, height: 40 } }),
        raw.image(sprite("badge"), { x: 4, y: 12, width: 12, height: 12 }),
        raw.image(sprite("extra"), { x: 20, y: 12, width: 10, height: 10 }),
    ],
});

const document: WindowDocument = {
    sprites: {
        badge: texture("window:gui/badge.png", { width: 12, height: 12 }),
        extra: texture("window:gui/extra", { width: 10, height: 10 }),
    },
    window: textured.windows[0]!,
};

export default document;
