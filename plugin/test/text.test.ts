import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { presets } from "../src/authoring/presets.ts";
import { smallCaps, smallCapsMinimessage } from "../src/authoring/text.ts";

test("industrial preset has the expected frame kinds", () => {
    const theme = presets.industrial();
    assert.equal(theme.frames?.["shell"]?.kind, "panel");
    assert.equal(theme.frames?.["tab_active"]?.kind, "button");
    assert.equal(theme.frames?.["button_confirm"]?.kind, "button");
    assert.equal(theme.frames?.["search_field"]?.kind, "panel");
    assert.equal(theme.frames?.["hud_panel"]?.kind, "panel");
    assert.equal(theme.sprites?.["pack_badge"]?.kind, "badge");
});

test("industrial preset options override colours", () => {
    const theme = presets.industrial({ badge_accent: "#123456" });
    assert.equal(theme.sprites?.["pack_badge"]?.accent_color, "#123456");
    assert.equal(presets.industrial().sprites?.["pack_badge"]?.accent_color, "#071a43");
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
