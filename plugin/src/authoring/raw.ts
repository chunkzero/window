/** The function-style authoring API, exported from `#plugins/window` as `raw`. See docs/AUTHORING.md. */
import { text as textElement } from "./elements.ts";
import * as textHelpers from "./text.ts";

export {
    anvilInput,
    button,
    choice,
    collection,
    column,
    flex,
    flex as box,
    grid,
    hotspot,
    hud,
    image,
    input,
    item,
    items,
    label,
    panel,
    region,
    repeater,
    row,
    section,
    show,
    slot,
    slotRects,
    sprite,
    spriteSlot,
    switchCase,
    switchCase as case,
    switchOn,
    toggle,
    ui,
} from "./elements.ts";

/** Static label text, or with a `text` handle dynamic text; also holds the `smallCaps` text helpers. */
export const text: typeof textElement & typeof textHelpers = Object.assign(
    (...args: Parameters<typeof textElement>) => textElement(...args),
    textHelpers,
);
