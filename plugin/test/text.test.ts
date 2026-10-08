import assert from "node:assert/strict";
import { test } from "node:test";

import { smallCaps, smallCapsMinimessage } from "../src/ui/text.ts";

test("small caps leaves non-letters alone", () => {
    assert.equal(smallCaps("Ab 1!é"), "ᴀʙ 1!é");
});

test("small caps minimessage skips tags", () => {
    assert.equal(smallCapsMinimessage("<gold>Hello</gold>"), "<gold>ʜᴇʟʟᴏ</gold>");
    assert.equal(smallCapsMinimessage("a <b"), "ᴀ <b");
});
