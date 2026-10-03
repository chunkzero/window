import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { presets } from "../src/authoring/presets.ts";
import { smallCaps, smallCapsMinimessage } from "../src/authoring/text.ts";

test("industrial preset has the expected frame kinds", () => {
    const theme = presets.industrial();
    assert.equal(theme.frames?.["shell"]?.kind, "panel");
    assert.equal(theme.frames?.["slot"]?.kind, "slot");
    assert.equal(theme.frames?.["button_confirm"]?.kind, "button");
    assert.equal(theme.frames?.["hud"]?.kind, "panel");
    assert.equal(theme.frames?.["hazard_bar"]?.kind, "hazard_bar");
    assert.equal(theme.sprites?.["rivet"]?.width, 5);
});

test("industrial preset derives bevels from overridden fills", () => {
    const theme = presets.industrial({ shell_fill: "#808080", highlight_color: "#123456" });
    const shell = theme.frames?.["shell"];
    assert.equal(shell?.fill, "#808080");
    assert.equal(shell?.highlight_color, "#71b2b9");
    assert.equal(shell?.shadow_color, "#4e5160");
    assert.equal(theme.frames?.["panel"]?.highlight_color, "#123456");
});

test("small caps leaves non-letters alone", () => {
    assert.equal(smallCaps("Ab 1!é"), "ᴀʙ 1!é");
});

test("small caps minimessage skips tags", () => {
    assert.equal(smallCapsMinimessage("<gold>Hello</gold>"), "<gold>ʜᴇʟʟᴏ</gold>");
    assert.equal(smallCapsMinimessage("a <b"), "ᴀ <b");
});

test("industrial preset matches the recorded theme", () => {
    const expected = JSON.parse(readFileSync(new URL("industrial.json", import.meta.url), "utf8"));
    assert.deepEqual(presets.industrial(), expected);
});
