import { raw } from "#plugins/window";
import type { WindowDocument } from "#plugins/window";

const textured = raw.ui({
    name: "textured",
    container: "generic_9x1",
    children: [
        raw.panel({ frame: "shell", x: 0, y: 0, width: 100, height: 40 }),
        raw.spriteSlot("badge", { x: 4, y: 12, width: 12, height: 12, sprite: "badge" }),
        raw.spriteSlot("extra", { x: 20, y: 12, width: 10, height: 10, sprite: "extra" }),
    ],
});

const document: WindowDocument = {
    theme: {
        frames: { shell: { texture: "window/frame.png", insets: 3 } },
        sprites: {
            badge: { texture: "window:gui/badge.png", width: 12, height: 12 },
            extra: { texture: "window:gui/extra", width: 10, height: 10 },
        },
    },
    window: textured.windows[0]!,
};

export default document;
