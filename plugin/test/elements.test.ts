import assert from "node:assert/strict";
import { test } from "node:test";

import * as window from "../src/authoring/index.ts";
import { Box, Container, Grid, Hotbar, Hotspot, Row, Sprite, Text } from "../src/authoring/jsx.ts";
import * as inventory from "../src/authoring/inventory.ts";

const { pattern, raw } = window;

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
    const button = raw.button("buy", {
        pattern: pattern.rect({ x: 0, y: 0, width: 1, height: 1 }),
    });
    assert.equal(button.type, "button");
    assert.equal(button.name, "buy");
    assert.equal(button.pattern?.kind, "rect");
});

test("collections can draw their slot frames natively", () => {
    const collection = raw.collection("results", {
        frame: "slot_cell",
        transform: { section: "container", x: 0, y: 0, width: 3, height: 1 },
    });
    assert.equal(collection.frame, "slot_cell");
});

test("toggle and choice constructors require their native states", () => {
    assert.throws(() => raw.toggle("quality", { width: 16, height: 16, states: {} as never }), /states\.on/);
    const choice = raw.choice("best", {
        width: 16,
        height: 16,
        states: { selected: {}, unselected: {} },
    });
    assert.equal(choice.type, "button");
});

test("anvil input preserves initial text and item model", () => {
    assert.deepEqual(raw.anvilInput("query", { initial: "maps", item_model: "window:gui/search" }), {
        type: "anvil_input",
        name: "query",
        initial: "maps",
        item_model: "window:gui/search",
    });
});

test("dynamic slots may defer width to automatic button layout", () => {
    const slot = raw.slot("caption", { bold: true });
    assert.equal(slot.name, "caption");
    assert.equal(slot.width, undefined);
});

test("visual constructors still reject inventory-irrelevant options", () => {
    assert.throws(
        () => raw.panel({ frame: "panel", width: 10, height: 10, slots: [0] } as never),
        /does not accept option `slots`/,
    );
});

test("repeater cell items accept a one-based cell_slot", () => {
    const item = raw.item("icon", { cell_slot: 2 });
    assert.deepEqual(item, { type: "item", name: "icon", cell_slot: 2 });
    assert.throws(() => raw.item("icon", { cell_slot: 1, slots: [0] }), /cannot be combined/);
    assert.throws(() => raw.item("icon", { cell_slot: 0 }), /positive integer/);
});

test("unknown option keys are rejected at runtime", () => {
    assert.throws(() => raw.row({ bogus: 1 } as never), /row does not accept option `bogus`/);
    assert.throws(() => raw.panel({ frame: "p", width: 1 } as never), /panel requires option `height`/);
});

test("switchOn and show build switch elements", () => {
    const buy = raw.label("Buy");
    assert.deepEqual(raw.switchOn("mode", { buy: { frame: "recess", children: [buy] }, sell: {} }, { x: 4, y: 6 }), {
        type: "switch",
        name: "mode",
        x: 4,
        y: 6,
        children: [
            { type: "case", value: "buy", frame: "recess", style: { direction: "column" }, children: [buy] },
            { type: "case", value: "sell", style: { direction: "column" }, children: [] },
        ],
    });
    const badge = raw.sprite("badge");
    assert.deepEqual(
        raw.show("on_sale", { x: 1, y: 2, children: [badge] }),
        raw.switchOn("on_sale", { true: { children: [badge] }, false: {} }, { x: 1, y: 2 }),
    );
    assert.throws(() => raw.switchOn("mode", {}), /requires at least one case/);
    assert.throws(() => raw.switchOn("mode", { buy: { width: 3 } as never }), /does not accept option `width`/);
    assert.deepEqual(raw.show("on_sale", { layout: { grow: 1 } }).layout, { grow: 1 });
});

test("flex, grid, and section build the same elements as their JSX components", () => {
    const fn = raw.section("container", {
        frame: "panel",
        children: [
            raw.flex({
                frame: "recess",
                style: { direction: "row", align: "center", padding: { left: 4, right: 4 } },
                layout: { column: { span: 9 } },
                children: [raw.slot("selection"), raw.slot("price", { width: 40, align: "right" })],
            }),
            raw.hotspot("info", {
                tooltip: "Info",
                layout: { column: { start: 1, span: 2 }, row: { start: 2, span: 1 } },
            }),
        ],
    });
    const jsx = Container({
        frame: "panel",
        children: [
            Row({
                span: 9,
                frame: "recess",
                padding: { left: 4, right: 4 },
                children: [Text({ bind: "selection" }), Text({ bind: "price", width: 40, align: "right" })],
            }),
            Hotspot({ name: "info", tooltip: "Info", span: [2, 1], at: [0, 1] }),
        ],
    });
    assert.deepEqual(fn, jsx);
    assert.deepEqual(
        raw.grid({
            style: { columns: 3, gap: 2 },
            children: [raw.sprite("a", { layout: { column: { span: 2 } } })],
        }),
        Grid({ columns: 3, gap: 2, children: Sprite({ name: "a", span: 2 }) }),
    );
    assert.deepEqual(raw.section("hotbar"), Hotbar({}));
    assert.deepEqual(raw.flex(), Box({}));
    assert.deepEqual(raw.grid(), Grid({}));
});

test("layout helpers reject options their element does not accept", () => {
    assert.throws(() => raw.flex({ gap: 2 } as never), /flex does not accept option `gap`/);
    assert.throws(() => raw.grid({ columns: 3 } as never), /grid does not accept option `columns`/);
    assert.throws(() => raw.section("container", { layout: {} } as never), /does not accept option `layout`/);
    assert.throws(() => raw.section("chest" as never), /section kind must be one of/);
    assert.throws(() => raw.anvilInput("q", { layout: {} } as never), /does not accept option `layout`/);
});
