/** The function-style authoring API, exported as `plugin:window/raw`. See docs/AUTHORING.md. */
import { text as textElement } from "../ui/elements.ts";
import * as textHelpers from "../ui/text.ts";

export {
    box,
    collection,
    hud,
    image,
    input,
    items,
    region,
    section,
    switchCase,
    switchCase as case,
    switchOn,
    ui,
} from "../ui/elements.ts";

/** Static label text, or with a `text` handle dynamic text; also holds the `smallCaps` text helpers. */
export const text: typeof textElement & typeof textHelpers = Object.assign(
    (...args: Parameters<typeof textElement>) => textElement(...args),
    textHelpers,
);
