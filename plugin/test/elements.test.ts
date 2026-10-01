import assert from "node:assert/strict";
import { test } from "node:test";

import * as window from "../src/authoring/index.ts";
import * as inventory from "../src/authoring/inventory.ts";

const { pattern } = window;

test("inventory aliases match", () => {
    assert.equal(window.pattern, inventory.pattern);
    assert.equal(window.containerSlot, inventory.containerSlot);
    assert.equal(window.playerSlots, inventory.playerSlots);
});

test("typed slot refs keep their backing area", () => {
    assert.deepEqual(window.containerSlot(7), { area: "container", index: 7 });
    assert.deepEqual(window.playerSlot(7), { area: "player", index: 7 });
});

test("inclusive ranges remain compact for Rust expansion", () => {
    assert.deepEqual(window.containerSlots(3, 8), [{ area: "container", first: 3, last: 8 }]);
});

test("descending ranges fail at the authoring boundary", () => {
    assert.throws(() => window.playerSlots(8, 3), /last must be greater than or equal to first/);
});

test("hotbar helpers reject invisible indices", () => {
    assert.throws(() => window.hotbarSlot(9), /between 0 and 8/);
    assert.throws(() => window.hotbarSlots(0, 9), /between 0 and 8/);
});

test("rect patterns preserve slot-space coordinates", () => {
    assert.deepEqual(pattern.rect({ section: "player", x: 1, y: 0, width: 3, height: 1 }), {
        kind: "rect",
        section: "player",
        x: 1,
        y: 0,
        width: 3,
        height: 1,
    });
});

test("explicit patterns reject empty slot lists", () => {
    assert.throws(() => pattern.slots([]), /requires at least one slot index/);
    assert.deepEqual(pattern.slots([1, 2]), { kind: "slots", section: "container", slots: [1, 2] });
});

test("grid patterns validate every dimension", () => {
    assert.throws(
        () => pattern.grid({ x: 0, y: 0, columns: 2, rows: 2, cell_width: 0, cell_height: 1 }),
        /cell_width must be a positive integer/,
    );
});

test("inventory patterns can drive visual controls without pixel coordinates", () => {
    const button = window.button("buy", {
        pattern: pattern.rect({ x: 0, y: 0, width: 1, height: 1 }),
    });
    assert.equal(button.type, "button");
    assert.equal(button.name, "buy");
    assert.equal(button.pattern?.kind, "rect");
});

test("collections can draw their slot frames natively", () => {
    const collection = window.collection("results", {
        frame: "slot_cell",
        transform: { section: "container", x: 0, y: 0, width: 3, height: 1 },
    });
    assert.equal(collection.frame, "slot_cell");
});

test("toggle and choice constructors require their native states", () => {
    assert.throws(() => window.toggle("quality", { width: 16, height: 16, states: {} as never }), /states\.on/);
    const choice = window.choice("best", {
        width: 16,
        height: 16,
        states: { selected: {}, unselected: {} },
    });
    assert.equal(choice.type, "button");
});

test("anvil input preserves initial text and item model", () => {
    assert.deepEqual(window.anvilInput("query", { initial: "maps", item_model: "window:gui/search" }), {
        type: "anvil_input",
        name: "query",
        initial: "maps",
        item_model: "window:gui/search",
    });
});

test("dynamic slots may defer width to automatic button layout", () => {
    const slot = window.slot("caption", { bold: true });
    assert.equal(slot.name, "caption");
    assert.equal(slot.width, undefined);
});

test("visual constructors still reject inventory-irrelevant options", () => {
    assert.throws(
        () => window.panel({ frame: "panel", width: 10, height: 10, slots: [0] } as never),
        /does not accept option `slots`/,
    );
});

test("repeater cell items accept a one-based cell_slot", () => {
    const item = window.item("icon", { cell_slot: 2 });
    assert.deepEqual(item, { type: "item", name: "icon", cell_slot: 2 });
    assert.throws(() => window.item("icon", { cell_slot: 1, slots: [0] }), /cannot be combined/);
    assert.throws(() => window.item("icon", { cell_slot: 0 }), /positive integer/);
    assert.throws(() => window.item("icon", {}), /requires `slots`, `pattern`, `transform`, or `cell_slot`/);
});

test("unknown option keys are rejected at runtime", () => {
    assert.throws(() => window.row({ bogus: 1 } as never), /row does not accept option `bogus`/);
    assert.throws(() => window.panel({ frame: "p", width: 1 } as never), /panel requires option `height`/);
});
